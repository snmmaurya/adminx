// crates/adminx/src/resource.rs - Fixed version with default implementations
use actix_web::{HttpRequest, HttpResponse, ResponseError};
use async_trait::async_trait;
use futures::future::BoxFuture;
use serde_json::{json, Value};
use crate::menu::{MenuItem, MenuAction};
use crate::actions::CustomAction;
use crate::nested::AdmixNestedResource;
use crate::error::AdminxError;
use crate::filters::parse_query;
use crate::pagination::PaginatedResponse;
use mongodb::{Collection, bson::{doc, oid::ObjectId, Document}};
use futures::TryStreamExt;

#[async_trait]
pub trait AdmixResource: Send + Sync {
    // ===========================
    // REQUIRED METHODS (Must be implemented)
    // ===========================
    fn new() -> Self where Self: Sized;
    fn resource_name(&self) -> &'static str;
    fn base_path(&self) -> &'static str;
    fn collection_name(&self) -> &'static str; 
    fn get_collection(&self) -> Collection<Document>;
    fn clone_box(&self) -> Box<dyn AdmixResource>;

    // ===========================
    // CONFIGURATION (Optional - with defaults)
    // ===========================

    /// Optional parent/super menu name to group this resource under.
    fn menu_group(&self) -> Option<&'static str> {
        None
    }

    /// Menu label for this resource (default: same as resource_name)
    fn menu(&self) -> &'static str {
        self.resource_name()
    }

    fn allowed_roles(&self) -> Vec<String> {
        vec!["admin".to_string()]
    }

    fn allowed_roles_with_permissions(&self) -> Value {
        json!({})
    }

    fn visible_fields_for_role(&self, _roles: &[String]) -> Vec<String> {
        vec![]
    }

    fn nested_resources(&self) -> Vec<Box<dyn AdmixNestedResource>> {
        vec![]
    }

    fn custom_actions(&self) -> Vec<CustomAction> {
        vec![]
    }

    fn allowed_actions(&self) -> Option<Vec<MenuAction>> {
        None // None means all actions are allowed
    }

    fn permit_keys(&self) -> Vec<&'static str> {
        vec![] // Override this to specify which fields can be created/updated
    }

    fn readonly_keys(&self) -> Vec<&'static str> {
        vec!["_id", "created_at", "updated_at"]
    }

    fn permit_filter_keys(&self) -> Vec<&'static str> {
        vec![] // Override this to specify which fields can be searched
    }

    // ===========================
    // UI STRUCTURE METHODS (Optional)
    // ===========================
    fn form_structure(&self) -> Option<Value> {
        None // Override to customize create/edit forms
    }

    fn list_structure(&self) -> Option<Value> {
        None // Override to customize list view
    }

    fn view_structure(&self) -> Option<Value> {
        None // Override to customize detail view
    }

    fn filters(&self) -> Option<Value> {
        None // Override to add search/filter functionality
    }

    // ===========================
    // DEFAULT CRUD IMPLEMENTATIONS
    // ===========================
    
    /// Default LIST implementation - override if you need custom logic
    fn list(&self, _req: &HttpRequest, query: String) -> BoxFuture<'static, HttpResponse> {
        let collection = self.get_collection();
        let resource_name = self.resource_name().to_string();
        
        Box::pin(async move {
            tracing::info!("Default list implementation for resource: {}", resource_name);
            
            let opts = parse_query(&query);
            
            // Get total count for pagination
            let total = match collection.count_documents(opts.filter.clone(), None).await {
                Ok(count) => count,
                Err(e) => {
                    tracing::error!("Error counting documents for {}: {}", resource_name, e);
                    return AdminxError::InternalError.error_response();
                }
            };
            
            // Build find options with sorting and pagination
            let mut find_options = mongodb::options::FindOptions::default();
            find_options.skip = Some(opts.skip);
            find_options.limit = Some(opts.limit as i64);
            if let Some(sort) = opts.sort {
                find_options.sort = Some(sort);
            }
            
            match collection.find(opts.filter, find_options).await {
                Ok(mut cursor) => {
                    let mut documents = Vec::new();
                    while let Some(doc) = cursor.try_next().await.unwrap_or(None) {
                        documents.push(doc);
                    }

                    tracing::info!("Found {} documents for {} out of {} total", 
                                 documents.len(), resource_name, total);
                    
                    HttpResponse::Ok().json(PaginatedResponse {
                        data: documents,
                        total,
                        page: (opts.skip / opts.limit) + 1,
                        per_page: opts.limit,
                    })
                }
                Err(e) => {
                    tracing::error!("Error executing find query for {}: {}", resource_name, e);
                    AdminxError::InternalError.error_response()
                }
            }
        })
    }

    /// Default GET implementation - override if you need custom logic
    fn get(&self, _req: &HttpRequest, id: String) -> BoxFuture<'static, HttpResponse> {
        let collection = self.get_collection();
        let resource_name = self.resource_name().to_string();
        
        Box::pin(async move {
            tracing::info!("Default get implementation for resource: {} with id: {}", resource_name, id);
            
            match ObjectId::parse_str(&id) {
                Ok(oid) => {
                    match collection.find_one(doc! { "_id": oid }, None).await {
                        Ok(Some(document)) => {
                            tracing::info!("Found document with id: {} for resource: {}", id, resource_name);
                            HttpResponse::Ok().json(document)
                        },
                        Ok(None) => {
                            tracing::warn!("Document not found with id: {} for resource: {}", id, resource_name);
                            AdminxError::NotFound.error_response()
                        },
                        Err(e) => {
                            tracing::error!("Database error getting document {} for {}: {}", id, resource_name, e);
                            AdminxError::InternalError.error_response()
                        }
                    }
                },
                Err(e) => {
                    tracing::error!("Invalid ObjectId {} for {}: {}", id, resource_name, e);
                    AdminxError::BadRequest("Invalid ID format".into()).error_response()
                }
            }
        })
    }

    /// Default CREATE implementation - override if you need custom logic
    fn create(&self, _req: &HttpRequest, payload: Value) -> BoxFuture<'static, HttpResponse> {
        let collection = self.get_collection();
        let permitted = self.permit_keys().into_iter().collect::<std::collections::HashSet<_>>();
        let resource_name = self.resource_name().to_string();
        
        Box::pin(async move {
            tracing::info!("Default create implementation for resource: {} with payload: {:?}", resource_name, payload);
            
            // Filter payload to only include permitted parameters
            let mut clean_map = serde_json::Map::new();
            if let Value::Object(map) = payload {
                for (key, value) in map {
                    if permitted.contains(key.as_str()) {
                        clean_map.insert(key, value);
                    }
                }
            }

            // Add timestamps
            let now = mongodb::bson::DateTime::now();
            clean_map.insert("created_at".to_string(), json!(now));
            clean_map.insert("updated_at".to_string(), json!(now));

            tracing::debug!("Cleaned payload for {}: {:?}", resource_name, clean_map);

            // Convert to BSON document
            match mongodb::bson::to_document(&Value::Object(clean_map)) {
                Ok(document) => {
                    match collection.insert_one(document, None).await {
                        Ok(insert_result) => {
                            tracing::info!("Document created successfully for {}: {:?}", resource_name, insert_result.inserted_id);
                            HttpResponse::Created().json(json!({
                                "success": true,
                                "message": format!("{} created successfully", resource_name),
                                "id": insert_result.inserted_id
                            }))
                        },
                        Err(e) => {
                            tracing::error!("Error inserting document for {}: {}", resource_name, e);
                            AdminxError::InternalError.error_response()
                        }
                    }
                },
                Err(e) => {
                    tracing::error!("Error converting payload to BSON for {}: {}", resource_name, e);
                    AdminxError::BadRequest("Invalid input data".into()).error_response()
                }
            }
        })
    }

    /// Default UPDATE implementation - override if you need custom logic
    fn update(&self, _req: &HttpRequest, id: String, payload: Value) -> BoxFuture<'static, HttpResponse> {
        let collection = self.get_collection();
        let permitted = self.permit_keys().into_iter().collect::<std::collections::HashSet<_>>();
        let resource_name = self.resource_name().to_string();
        
        Box::pin(async move {
            tracing::info!("Default update implementation for resource: {} with id: {} and payload: {:?}", 
                         resource_name, id, payload);
            
            match ObjectId::parse_str(&id) {
                Ok(oid) => {
                    // Filter payload to only include permitted parameters
                    let mut clean_map = serde_json::Map::new();
                    if let Value::Object(map) = payload {
                        for (key, value) in map {
                            if permitted.contains(key.as_str()) {
                                clean_map.insert(key, value);
                            }
                        }
                    }

                    // Add updated timestamp
                    clean_map.insert("updated_at".to_string(), json!(mongodb::bson::DateTime::now()));

                    let bson_payload: Document = match mongodb::bson::to_document(&Value::Object(clean_map)) {
                        Ok(doc) => doc,
                        Err(e) => {
                            tracing::error!("Error converting payload to BSON for {}: {}", resource_name, e);
                            return AdminxError::BadRequest("Invalid payload format".into()).error_response();
                        }
                    };

                    let update_doc = doc! { "$set": bson_payload };

                    match collection.update_one(doc! { "_id": oid }, update_doc, None).await {
                        Ok(result) => {
                            if result.modified_count > 0 {
                                tracing::info!("Document {} updated successfully for {}", id, resource_name);
                                HttpResponse::Ok().json(json!({
                                    "success": true,
                                    "message": format!("{} updated successfully", resource_name),
                                    "modified_count": result.modified_count
                                }))
                            } else {
                                tracing::warn!("No document found to update with id: {} for {}", id, resource_name);
                                AdminxError::NotFound.error_response()
                            }
                        },
                        Err(e) => {
                            tracing::error!("Error updating document {} for {}: {}", id, resource_name, e);
                            AdminxError::InternalError.error_response()
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Invalid ObjectId {} for {}: {}", id, resource_name, e);
                    AdminxError::BadRequest("Invalid ID format".into()).error_response()
                }
            }
        })
    }

    /// Default DELETE implementation - override if you need custom logic
    fn delete(&self, _req: &HttpRequest, id: String) -> BoxFuture<'static, HttpResponse> {
        let collection = self.get_collection();
        let resource_name = self.resource_name().to_string();
        
        Box::pin(async move {
            tracing::info!("Default delete implementation for resource: {} with id: {}", resource_name, id);
            
            match ObjectId::parse_str(&id) {
                Ok(oid) => {
                    match collection.delete_one(doc! { "_id": oid }, None).await {
                        Ok(result) => {
                            if result.deleted_count > 0 {
                                tracing::info!("Document {} deleted successfully for {}", id, resource_name);
                                HttpResponse::Ok().json(json!({
                                    "success": true,
                                    "message": format!("{} deleted successfully", resource_name),
                                    "deleted_count": result.deleted_count
                                }))
                            } else {
                                tracing::warn!("No document found to delete with id: {} for {}", id, resource_name);
                                AdminxError::NotFound.error_response()
                            }
                        },
                        Err(e) => {
                            tracing::error!("Error deleting document {} for {}: {}", id, resource_name, e);
                            AdminxError::InternalError.error_response()
                        }
                    }
                },
                Err(e) => {
                    tracing::error!("Invalid ObjectId {} for {}: {}", id, resource_name, e);
                    AdminxError::BadRequest("Invalid ID format".into()).error_response()
                }
            }
        })
    }

    // ===========================
    // MENU GENERATION (Optional Override)
    // ===========================
    fn generate_menu(&self) -> Option<MenuItem> {
        // Build a resource node with NO action children.
        let resource_node = MenuItem {
            title: self.menu().to_string(),
            path: self.base_path().to_string(),
            icon: Some("users".to_string()),
            order: Some(10),
            children: None, // <-- no List/Create/View/Edit/Delete
        };

        // If a parent menu is set, wrap the resource under it.
        if let Some(group_title) = self.menu_group() {
            return Some(MenuItem {
                title: group_title.to_string(),
                path: String::new(),            // non-clickable parent
                icon: None,
                order: Some(10),
                children: Some(vec![resource_node]),
            });
        }

        // Otherwise, just the resource node at top level.
        Some(resource_node)
    }


    fn build_adminx_menus(&self) -> Option<MenuItem> {
        self.generate_menu()
    }
}

// Manual clone implementation
impl Clone for Box<dyn AdmixResource> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}