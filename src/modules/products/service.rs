use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    modules::products::{
        dto::{
            CreateProductRequest, ProductQuery, ProductResponse, ProductsListResponse,
            UpdateProductRequest,
        },
        repository::ProductRepository,
    },
    shared::{extractor::AuthenticatedUser, money::validate_amount, types::UserRole},
};

#[derive(Clone)]
pub struct ProductService {
    repository: ProductRepository,
}

impl ProductService {
    pub fn new(repository: ProductRepository) -> Self {
        Self { repository }
    }

    pub async fn create_product(
        &self,
        actor: &AuthenticatedUser,
        request: CreateProductRequest,
    ) -> Result<ProductResponse, AppError> {
        ensure_admin(actor)?;
        validate_amount(request.price_amount)?;

        if self.repository.find_by_sku(&request.sku).await?.is_some() {
            return Err(AppError::Conflict("sku already exists".into()));
        }

        ProductResponse::from_model(self.repository.create(&request).await?)
    }

    pub async fn list_products(&self, query: ProductQuery) -> Result<ProductsListResponse, AppError> {
        let page = query.page.max(1);
        let limit = query.limit.clamp(1, 100);
        let items = self
            .repository
            .list(page, limit, query.active_only)
            .await?
            .into_iter()
            .map(ProductResponse::from_model)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ProductsListResponse { items, page, limit })
    }

    pub async fn get_product(&self, product_id: Uuid) -> Result<ProductResponse, AppError> {
        let product = self
            .repository
            .find_by_id(product_id)
            .await?
            .ok_or_else(|| AppError::NotFound("product was not found".into()))?;
        ProductResponse::from_model(product)
    }

    pub async fn update_product(
        &self,
        actor: &AuthenticatedUser,
        product_id: Uuid,
        request: UpdateProductRequest,
    ) -> Result<ProductResponse, AppError> {
        ensure_admin(actor)?;
        if let Some(amount) = request.price_amount {
            validate_amount(amount)?;
        }
        if let Some(sku) = request.sku.as_deref()
            && let Some(existing) = self.repository.find_by_sku(sku).await?
            && existing.id != product_id
        {
            return Err(AppError::Conflict("sku already exists".into()));
        }

        let product = self
            .repository
            .update(product_id, request)
            .await?
            .ok_or_else(|| AppError::NotFound("product was not found".into()))?;
        ProductResponse::from_model(product)
    }

    pub async fn delete_product(
        &self,
        actor: &AuthenticatedUser,
        product_id: Uuid,
    ) -> Result<(), AppError> {
        ensure_admin(actor)?;
        if !self.repository.delete(product_id).await? {
            return Err(AppError::NotFound("product was not found".into()));
        }
        Ok(())
    }
}

fn ensure_admin(actor: &AuthenticatedUser) -> Result<(), AppError> {
    if actor.role == UserRole::Admin {
        Ok(())
    } else {
        Err(AppError::Forbidden("admin access is required".into()))
    }
}
