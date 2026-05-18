use chrono::{DateTime, Utc};

pub struct ReceiptPdfLineItem<'a> {
    pub name: &'a str,
    pub quantity: i32,
    pub unit_price_amount: i64,
    pub line_total_amount: i64,
}

pub struct ReceiptPdfInput<'a> {
    pub receipt_number: &'a str,
    pub issued_at: DateTime<Utc>,
    pub customer_name: &'a str,
    pub customer_email: &'a str,
    pub order_id: &'a str,
    pub currency: &'a str,
    pub total_amount: i64,
    pub payment_reference: Option<&'a str>,
    pub items: &'a [ReceiptPdfLineItem<'a>],
}

pub fn render_receipt_pdf(input: &ReceiptPdfInput<'_>) -> Vec<u8> {
    let mut lines = vec![
        "Receipt".to_string(),
        format!("Receipt Number: {}", input.receipt_number),
        format!("Issued At: {}", input.issued_at.to_rfc3339()),
        format!(
            "Customer: {} <{}>",
            input.customer_name, input.customer_email
        ),
        format!("Order ID: {}", input.order_id),
        format!("Currency: {}", input.currency.to_uppercase()),
        String::new(),
        "Items".to_string(),
    ];

    for item in input.items {
        lines.push(format!(
            "- {} x{} @ {} = {}",
            item.name, item.quantity, item.unit_price_amount, item.line_total_amount
        ));
    }

    lines.push(String::new());
    lines.push(format!("Total: {}", input.total_amount));
    if let Some(reference) = input.payment_reference {
        lines.push(format!("Payment Reference: {reference}"));
    }

    let text = lines.join("\n");
    build_basic_pdf(&text)
}

fn build_basic_pdf(text: &str) -> Vec<u8> {
    let escaped = text
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)");
    let stream = format!(
        "BT /F1 12 Tf 50 780 Td 14 TL ({}) Tj T* ET",
        escaped.replace('\n', ") Tj T* (")
    );
    let mut objects = Vec::new();
    objects.push("1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n".to_string());
    objects.push("2 0 obj << /Type /Pages /Count 1 /Kids [3 0 R] >> endobj\n".to_string());
    objects.push(
        "3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >> endobj\n"
            .to_string(),
    );
    objects.push(format!(
        "4 0 obj << /Length {} >> stream\n{}\nendstream endobj\n",
        stream.len(),
        stream
    ));
    objects.push(
        "5 0 obj << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> endobj\n".to_string(),
    );

    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = vec![0usize];
    for object in &objects {
        offsets.push(pdf.len());
        pdf.extend_from_slice(object.as_bytes());
    }

    let xref_offset = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n", offsets.len()).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets.iter().skip(1) {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer << /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            offsets.len(),
            xref_offset
        )
        .as_bytes(),
    );
    pdf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_pdf_header() {
        let bytes = render_receipt_pdf(&ReceiptPdfInput {
            receipt_number: "RCT-1",
            issued_at: Utc::now(),
            customer_name: "Demo",
            customer_email: "demo@example.com",
            order_id: "order-1",
            currency: "thb",
            total_amount: 1000,
            payment_reference: None,
            items: &[],
        });

        assert!(bytes.starts_with(b"%PDF-1.4"));
    }
}
