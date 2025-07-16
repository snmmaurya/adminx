// adminx/src/resource.rs
use actix_web::{web, HttpRequest, HttpResponse};
use async_trait::async_trait;
use futures::future::BoxFuture;
use serde_json::{json, Value};
use crate::menu::{MenuItem, MenuAction};
use crate::actions::CustomAction;
use crate::nested::AdmixNestedResource;
use crate::schemas::adminx_schema::AdminxSchema;

#[async_trait]
pub trait AdmixResource: Send + Sync {
    fn resource_name(&self) -> &'static str;
    fn allowed_roles(&self) -> Vec<String>;

    fn allowed_roles_with_permissions(&self) -> Value {
        json!({})
    }

    fn visible_fields_for_role(&self, roles: &[String]) -> Vec<String> {
        vec![]
    }

    fn nested_resources(&self) -> Vec<Box<dyn AdmixNestedResource>> {
        vec![]
    }

    fn custom_actions(&self) -> Vec<CustomAction> {
        vec![]
    }

    fn new() -> Self where Self: Sized;
    fn base_path(&self) -> &'static str;
    // fn register_routes(&self, cfg: &mut web::ServiceConfig);

    fn allowed_actions(&self) -> Option<Vec<MenuAction>> {
        None
    }

    fn permit_params(&self) -> Vec<&'static str> {
        vec![]
    }

    fn readonly_fields(&self) -> Vec<&'static str> {
        vec![]
    }

    fn permit_query_fields(&self) -> Vec<&'static str> {
        vec![]
    }

    /// For cloning trait objects
    fn clone_box(&self) -> Box<dyn AdmixResource>;

    fn list(&self, req: &HttpRequest, query: String) -> BoxFuture<'static, HttpResponse>;
    fn get(&self, req: &HttpRequest, id: String) -> BoxFuture<'static, HttpResponse>;
    fn create(&self, req: &HttpRequest, payload: Value) -> BoxFuture<'static, HttpResponse>;
    fn update(&self, req: &HttpRequest, id: String, payload: Value) -> BoxFuture<'static, HttpResponse>;
    fn delete(&self, req: &HttpRequest, id: String) -> BoxFuture<'static, HttpResponse>;

    // # Deprecated
    fn generate_menu(&self) -> Option<MenuItem> {
        let actions = self.allowed_actions().unwrap_or_else(|| {
            vec![
                MenuAction::List,
                MenuAction::Create,
                MenuAction::View,
                MenuAction::Edit,
                MenuAction::Delete,
            ]
        });

        Some(MenuItem {
            title: self.resource_name().to_string(),
            path: self.base_path().to_string(),
            icon: Some("users".to_string()), // Optional
            order: Some(10),
            children: Some(
                actions.into_iter().map(|action| MenuItem {
                    title: format!("{} {}", action.as_str().to_uppercase(), self.resource_name()),
                    path: action.to_path(self.base_path()),
                    children: None,
                    icon: None,
                    order: None,
                }).collect()
            ),
        })
    }



    fn build_adminx_menus(&self) -> Option<MenuItem> {
        let actions = self.allowed_actions().unwrap_or_else(|| {
            vec![
                MenuAction::List,
                MenuAction::Create,
                MenuAction::View,
                MenuAction::Edit,
                MenuAction::Delete,
            ]
        });

        Some(MenuItem {
            title: self.resource_name().to_string(),
            path: self.base_path().to_string(),
            icon: Some("users".to_string()), // Customize as needed
            order: Some(10),
            children: Some(
                actions.into_iter().map(|action| MenuItem {
                    title: format!("{} {}", action.as_str().to_uppercase(), self.resource_name()),
                    path: action.to_path(self.base_path()),
                    children: None,
                    icon: None,
                    order: None,
                }).collect()
            ),
        })
    }


    // fn form_structure(&self) -> Option<Value> {
    //     Some(json!({
    //         "groups": [
    //             {
    //                 "title": "Details",
    //                 "fields": ["username", "email"]
    //             },
    //             {
    //                 "title": "Meta",
    //                 "fields": ["status"]
    //             }
    //         ]
    //     }))
    // }

    fn form_structure(&self) -> Option<Value>;

    // fn list_structure(&self) -> Option<Value> {
    //     Some(json!({
    //         "columns": [
    //             { "field": "username", "label": "Username" },
    //             { "field": "email", "label": "Email" },
    //             { "field": "status", "label": "Status" }
    //         ],
    //         "actions": ["view", "edit", "delete"]
    //     }))
    // }

    fn list_structure(&self) -> Option<Value>;

    // fn view_structure(&self) -> Option<Value> {
    //     Some(json!({
    //         "sections": [
    //             {
    //                 "title": "User Info",
    //                 "fields": ["username", "email"]
    //             },
    //             {
    //                 "title": "System Info",
    //                 "fields": ["status", "created_at", "updated_at"]
    //             }
    //         ]
    //     }))
    // }

    fn view_structure(&self) -> Option<Value>;

    // fn list_filters(&self) -> Option<Value> {
    //     Some(json!({
    //         "filters": [
    //             {
    //                 "field": "status",
    //                 "type": "select",
    //                 "options": ["Active", "Inactive"]
    //             },
    //             {
    //                 "field": "email",
    //                 "type": "text"
    //             }
    //         ]
    //     }))
    // }

    fn list_filters(&self) -> Option<Value>;
}

// ✅ Manual clone implementation
impl Clone for Box<dyn AdmixResource> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
