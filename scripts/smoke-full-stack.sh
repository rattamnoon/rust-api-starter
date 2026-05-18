#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"
COMPOSE_CMD=(docker compose -f docker-compose.yml)

BASE_URL="${BASE_URL:-http://127.0.0.1:8080}"
STRIPE_WEBHOOK_SECRET="${STRIPE_WEBHOOK_SECRET:-local-stripe-webhook-secret}"
SMOKE_RUN_ID="${SMOKE_RUN_ID:-$(date +%s)}"
ADMIN_EMAIL="${ADMIN_EMAIL:-smoke-admin-${SMOKE_RUN_ID}@example.com}"
ADMIN_PASSWORD="${ADMIN_PASSWORD:-password123}"
ADMIN_NAME="${ADMIN_NAME:-Smoke Admin}"

wait_for_url() {
  local url="$1"
  local name="$2"
  for _ in $(seq 1 60); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      echo "ready: $name"
      return 0
    fi
    sleep 2
  done
  echo "timeout waiting for $name ($url)" >&2
  return 1
}

json_field() {
  local key="$1"
  python3 -c 'import json,sys; print(json.load(sys.stdin).get(sys.argv[1], ""))' "$key"
}

compose_psql() {
  "${COMPOSE_CMD[@]}" exec -T postgres psql -U postgres -d app -tAqc "$1"
}

echo "starting stack"
"${COMPOSE_CMD[@]}" up -d --build

wait_for_url "${BASE_URL}/api/v1/health" "api"
wait_for_url "http://127.0.0.1:9090/-/healthy" "prometheus"
wait_for_url "http://127.0.0.1:3000/api/health" "grafana"
wait_for_url "http://127.0.0.1:8081/actuator/health" "kafka-ui"

echo "registering admin user"
REGISTER_RESPONSE="$(
  curl -fsS -X POST "${BASE_URL}/api/v1/auth/register" \
    -H 'Content-Type: application/json' \
    -d "{\"email\":\"${ADMIN_EMAIL}\",\"password\":\"${ADMIN_PASSWORD}\",\"name\":\"${ADMIN_NAME}\"}"
)"

compose_psql "UPDATE users SET role = 'admin' WHERE email = '${ADMIN_EMAIL}';" >/dev/null

LOGIN_RESPONSE="$(
  curl -fsS -X POST "${BASE_URL}/api/v1/auth/login" \
    -H 'Content-Type: application/json' \
    -d "{\"email\":\"${ADMIN_EMAIL}\",\"password\":\"${ADMIN_PASSWORD}\"}"
)"
ACCESS_TOKEN="$(printf '%s' "${LOGIN_RESPONSE}" | json_field access_token)"

echo "creating product"
PRODUCT_RESPONSE="$(
  curl -fsS -X POST "${BASE_URL}/api/v1/products" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H 'Content-Type: application/json' \
    -d '{"sku":"SMOKE-001","name":"Smoke Product","description":"Smoke test product","price_amount":19900,"currency":"thb","is_active":true}'
)"
PRODUCT_ID="$(printf '%s' "${PRODUCT_RESPONSE}" | json_field id)"

echo "creating order"
ORDER_RESPONSE="$(
  curl -fsS -X POST "${BASE_URL}/api/v1/orders" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H 'Content-Type: application/json' \
    -d "{\"items\":[{\"product_id\":\"${PRODUCT_ID}\",\"quantity\":1}]}"
)"
ORDER_ID="$(printf '%s' "${ORDER_RESPONSE}" | json_field id)"

echo "simulating stripe webhook"
WEBHOOK_BODY="$(cat <<JSON
{"id":"evt_smoke_${SMOKE_RUN_ID}","type":"checkout.session.completed","data":{"object":{"id":"cs_smoke_${SMOKE_RUN_ID}","payment_intent":"pi_smoke_${SMOKE_RUN_ID}","payment_status":"paid","amount_total":19900,"currency":"thb","metadata":{"order_id":"${ORDER_ID}"}}}}
JSON
)"
TIMESTAMP="$(date +%s)"
SIGNATURE_HASH="$(printf '%s' "${TIMESTAMP}.${WEBHOOK_BODY}" | openssl dgst -sha256 -hmac "${STRIPE_WEBHOOK_SECRET}" | awk '{print $NF}')"
STRIPE_SIGNATURE="t=${TIMESTAMP},v1=${SIGNATURE_HASH}"

WEBHOOK_RESPONSE="$(
  curl -fsS -X POST "${BASE_URL}/api/v1/payments/webhooks/stripe" \
    -H "Stripe-Signature: ${STRIPE_SIGNATURE}" \
    -H 'Content-Type: application/json' \
    -d "${WEBHOOK_BODY}"
)"
JOB_ID="$(printf '%s' "${WEBHOOK_RESPONSE}" | json_field job_id)"

echo "waiting for receipt flow"
for _ in $(seq 1 60); do
  RECEIPT_STATUS="$(compose_psql "SELECT status FROM receipts ORDER BY created_at DESC LIMIT 1;")"
  if [[ "${RECEIPT_STATUS}" == "emailed" ]]; then
    break
  fi
  sleep 2
done

FINAL_RECEIPT_STATUS="$(compose_psql "SELECT status FROM receipts ORDER BY created_at DESC LIMIT 1;")"
if [[ "${FINAL_RECEIPT_STATUS}" != "emailed" ]]; then
  echo "receipt flow did not finish successfully, status=${FINAL_RECEIPT_STATUS}" >&2
  exit 1
fi

echo "checking database state"
ORDER_STATUS="$(compose_psql "SELECT status FROM orders WHERE id = '${ORDER_ID}';")"
[[ "${ORDER_STATUS}" == "paid" ]] || { echo "expected order paid, got ${ORDER_STATUS}" >&2; exit 1; }

RECEIPT_ID="$(compose_psql "SELECT id FROM receipts WHERE order_id = '${ORDER_ID}';")"
EMAIL_DELIVERY_STATUS="$(compose_psql "SELECT status FROM email_deliveries WHERE receipt_id = '${RECEIPT_ID}' ORDER BY created_at DESC LIMIT 1;")"
[[ "${EMAIL_DELIVERY_STATUS}" == "sent" ]] || { echo "expected sent email delivery, got ${EMAIL_DELIVERY_STATUS}" >&2; exit 1; }

PDF_COUNT="$(find uploads/receipts -type f -name '*.pdf' 2>/dev/null | wc -l | tr -d ' ')"
[[ "${PDF_COUNT}" != "0" ]] || { echo "expected receipt pdf in uploads/receipts" >&2; exit 1; }

echo "checking kafka topics"
USERS_EVENTS="$("${COMPOSE_CMD[@]}" exec -T kafka kafka-console-consumer.sh --bootstrap-server localhost:9092 --topic users.events --from-beginning --timeout-ms 5000 2>/dev/null || true)"
ORDERS_EVENTS="$("${COMPOSE_CMD[@]}" exec -T kafka kafka-console-consumer.sh --bootstrap-server localhost:9092 --topic orders.events --from-beginning --timeout-ms 5000 2>/dev/null || true)"
RECEIPTS_EVENTS="$("${COMPOSE_CMD[@]}" exec -T kafka kafka-console-consumer.sh --bootstrap-server localhost:9092 --topic receipts.events --from-beginning --timeout-ms 5000 2>/dev/null || true)"

printf '%s' "${USERS_EVENTS}" | grep -q '"event_type":"user.registered"' || { echo "missing user.registered kafka event" >&2; exit 1; }
printf '%s' "${ORDERS_EVENTS}" | grep -q '"event_type":"order.paid"' || { echo "missing order.paid kafka event" >&2; exit 1; }
printf '%s' "${RECEIPTS_EVENTS}" | grep -q '"event_type":"receipt.generated"' || { echo "missing receipt.generated kafka event" >&2; exit 1; }
printf '%s' "${RECEIPTS_EVENTS}" | grep -q '"event_type":"receipt.emailed"' || { echo "missing receipt.emailed kafka event" >&2; exit 1; }

echo "checking job states"
FAILED_JOBS="$(compose_psql "SELECT COUNT(*) FROM jobs WHERE status IN ('failed', 'dead_lettered');")"
[[ "${FAILED_JOBS}" == "0" ]] || { echo "expected no failed jobs, got ${FAILED_JOBS}" >&2; exit 1; }
SUCCESSFUL_CHAIN="$(compose_psql "SELECT COUNT(*) FROM jobs WHERE id = '${JOB_ID}' OR job_type = 'send_receipt_email';")"
[[ "${SUCCESSFUL_CHAIN}" -ge 2 ]] || { echo "expected receipt job chain to be present" >&2; exit 1; }

echo "smoke flow passed"
