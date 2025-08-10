// crates/adminx/src/helpers/asset_helper.rs
use actix_web::{web, HttpResponse, Result};
use rust_embed::RustEmbed;
use log::{info, warn, debug};

#[derive(RustEmbed)]
#[folder = "assets/"]
struct AdminXAssets;

async fn serve_embedded_asset(path: web::Path<String>) -> Result<HttpResponse> {
    let file_path = path.into_inner();
    debug!("🔍 AdminX: Requested asset: {}", file_path);
    
    if let Some(content) = AdminXAssets::get(&file_path) {
        let mime_type = mime_guess::from_path(&file_path).first_or_octet_stream();
        debug!("✅ AdminX: Found asset: {} (mime: {})", file_path, mime_type);
        
        Ok(HttpResponse::Ok()
            .content_type(mime_type.as_ref())
            .body(content.data.into_owned()))
    } else {
        warn!("❌ AdminX: Asset not found: {}", file_path);
        
        // List available assets for debugging
        debug!("📦 Available AdminX assets:");
        for file in AdminXAssets::iter() {
            debug!("  - {}", file);
        }
        
        Ok(HttpResponse::NotFound().body(format!("AdminX asset not found: {}", file_path)))
    }
}

async fn debug_adminx_assets() -> Result<HttpResponse> {
    let mut assets = Vec::new();
    for file in AdminXAssets::iter() {
        assets.push(file.to_string());
    }
    
    let response = format!(
        "AdminX Embedded Assets:\n\n{}\n\nTry: /assets/images/AX.png",
        assets.join("\n")
    );
    
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(response))
}

pub fn mount_adminx_static(cfg: &mut web::ServiceConfig) {
    info!("🔧 AdminX: Mounting static assets at /assets/{{path:.*}}");
    
    // Add debug route to see what assets are available
    cfg.route("/adminx-assets-debug", web::get().to(debug_adminx_assets));
    
    // Mount the embedded asset handler
    cfg.route("/assets/{path:.*}", web::get().to(serve_embedded_asset));
    
    info!("✅ AdminX: Static asset routes registered");
}