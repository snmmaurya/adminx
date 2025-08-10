// // src/bin/adminx.rs
// use clap::{Arg, Command, ArgMatches};
// use std::io::{self, Write};
// use mongodb::bson::{doc, oid::ObjectId, DateTime as BsonDateTime};
// use bcrypt::{hash, DEFAULT_COST};
// use anyhow::{Result, anyhow};
// use futures::stream::TryStreamExt;

// // Import your AdminX modules
// use adminx::{
//     models::adminx_model::{AdminxUser, get_admin_by_email, get_all_admins, delete_admin_by_id, update_admin_status},
//     utils::{
//         database::{get_adminx_database, initiate_database},
//         auth::AdminxStatus,
//     },
//     configs::initializer::{get_adminx_config, setup_adminx_logging},
// };

// #[tokio::main]
// async fn main() -> Result<()> {
//     // Load configuration and setup logging
//     let config = get_adminx_config();
//     setup_adminx_logging(&config);

//     // Initialize database connection
//     let db = get_adminx_database();
//     initiate_database(db.clone());

//     let app = Command::new("adminx")
//         .version("1.0.0")
//         .author("Your Name <your.email@example.com>")
//         .about("AdminX CLI - Manage admin users for your AdminX application")
//         .subcommand(
//             Command::new("user")
//                 .about("User management commands")
//                 .subcommand(
//                     Command::new("create")
//                         .about("Create a new admin user")
//                         .arg(
//                             Arg::new("email")
//                                 .short('e')
//                                 .long("email")
//                                 .value_name("EMAIL")
//                                 .help("Admin user email address")
//                                 .required(false),
//                         )
//                         .arg(
//                             Arg::new("username")
//                                 .short('u')
//                                 .long("username")
//                                 .value_name("USERNAME")
//                                 .help("Admin username")
//                                 .required(false),
//                         )
//                         .arg(
//                             Arg::new("password")
//                                 .short('p')
//                                 .long("password")
//                                 .value_name("PASSWORD")
//                                 .help("Admin password (will prompt if not provided)")
//                                 .required(false),
//                         )
//                         .arg(
//                             Arg::new("status")
//                                 .short('s')
//                                 .long("status")
//                                 .value_name("STATUS")
//                                 .help("User status (active, inactive, suspended)")
//                                 .default_value("active")
//                                 .value_parser(["active", "inactive", "suspended"]),
//                         )
//                         .arg(
//                             Arg::new("interactive")
//                                 .short('i')
//                                 .long("interactive")
//                                 .help("Interactive mode - prompt for all fields")
//                                 .action(clap::ArgAction::SetTrue),
//                         ),
//                 )
//                 .subcommand(
//                     Command::new("list")
//                         .about("List all admin users")
//                         .arg(
//                             Arg::new("include-deleted")
//                                 .long("include-deleted")
//                                 .help("Include soft-deleted users in the list")
//                                 .action(clap::ArgAction::SetTrue),
//                         )
//                         .arg(
//                             Arg::new("format")
//                                 .short('f')
//                                 .long("format")
//                                 .value_name("FORMAT")
//                                 .help("Output format")
//                                 .default_value("table")
//                                 .value_parser(["table", "json", "csv"]),
//                         ),
//                 )
//                 .subcommand(
//                     Command::new("delete")
//                         .about("Soft delete an admin user")
//                         .arg(
//                             Arg::new("email")
//                                 .short('e')
//                                 .long("email")
//                                 .value_name("EMAIL")
//                                 .help("Email of the user to delete")
//                                 .required(true),
//                         )
//                         .arg(
//                             Arg::new("confirm")
//                                 .long("confirm")
//                                 .help("Skip confirmation prompt")
//                                 .action(clap::ArgAction::SetTrue),
//                         ),
//                 )
//                 .subcommand(
//                     Command::new("status")
//                         .about("Update user status")
//                         .arg(
//                             Arg::new("email")
//                                 .short('e')
//                                 .long("email")
//                                 .value_name("EMAIL")
//                                 .help("Email of the user to update")
//                                 .required(true),
//                         )
//                         .arg(
//                             Arg::new("status")
//                                 .short('s')
//                                 .long("status")
//                                 .value_name("STATUS")
//                                 .help("New status")
//                                 .required(true)
//                                 .value_parser(["active", "inactive", "suspended"]),
//                         ),
//                 )
//                 .subcommand(
//                     Command::new("info")
//                         .about("Show detailed information about a user")
//                         .arg(
//                             Arg::new("email")
//                                 .short('e')
//                                 .long("email")
//                                 .value_name("EMAIL")
//                                 .help("Email of the user to show info for")
//                                 .required(true),
//                         ),
//                 ),
//         )
//         .subcommand(
//             Command::new("db")
//                 .about("Database management commands")
//                 .subcommand(
//                     Command::new("stats")
//                         .about("Show database statistics"),
//                 ),
//         );

//     let matches = app.get_matches();

//     match matches.subcommand() {
//         Some(("user", user_matches)) => handle_user_commands(user_matches).await?,
//         Some(("db", db_matches)) => handle_db_commands(db_matches).await?,
//         _ => {
//             println!("No command specified. Use --help for usage information.");
//         }
//     }

//     Ok(())
// }

// async fn handle_user_commands(matches: &ArgMatches) -> Result<()> {
//     match matches.subcommand() {
//         Some(("create", create_matches)) => create_user(create_matches).await?,
//         Some(("list", list_matches)) => list_users(list_matches).await?,
//         Some(("delete", delete_matches)) => delete_user(delete_matches).await?,
//         Some(("status", status_matches)) => update_user_status(status_matches).await?,
//         Some(("info", info_matches)) => show_user_info(info_matches).await?,
//         _ => println!("Unknown user command. Use 'adminx user --help' for usage."),
//     }
//     Ok(())
// }

// async fn handle_db_commands(matches: &ArgMatches) -> Result<()> {
//     match matches.subcommand() {
//         Some(("stats", _)) => show_db_stats().await?,
//         _ => println!("Unknown db command. Use 'adminx db --help' for usage."),
//     }
//     Ok(())
// }

// async fn create_user(matches: &ArgMatches) -> Result<()> {
//     let interactive = matches.get_flag("interactive");
    
//     let email = if interactive || matches.get_one::<String>("email").is_none() {
//         prompt_input("Email address")?
//     } else {
//         matches.get_one::<String>("email").unwrap().clone()
//     };

//     // Check if user already exists
//     if let Some(_existing) = get_admin_by_email(&email).await {
//         return Err(anyhow!("❌ User with email '{}' already exists", email));
//     }

//     let username = if interactive || matches.get_one::<String>("username").is_none() {
//         prompt_input("Username")?
//     } else {
//         matches.get_one::<String>("username").unwrap().clone()
//     };

//     let password = if interactive || matches.get_one::<String>("password").is_none() {
//         prompt_password("Password")?
//     } else {
//         matches.get_one::<String>("password").unwrap().clone()
//     };

//     let status_str = if interactive {
//         prompt_select("Status", &["active", "inactive", "suspended"], "active")?
//     } else {
//         matches.get_one::<String>("status").unwrap().clone()
//     };

//     let status = match status_str.as_str() {
//         "active" => AdminxStatus::Active,
//         "inactive" => AdminxStatus::Inactive,
//         "suspended" => AdminxStatus::Suspended,
//         _ => return Err(anyhow!("Invalid status: {}", status_str)),
//     };

//     // Hash the password
//     let hashed_password = hash(&password, DEFAULT_COST)
//         .map_err(|e| anyhow!("Failed to hash password: {}", e))?;

//     // Create the user
//     let now = BsonDateTime::now();
//     let new_user = AdminxUser {
//         id: None, // MongoDB will generate this
//         username,
//         email: email.clone(),
//         password: hashed_password,
//         delete: false,
//         status,
//         created_at: now,
//         updated_at: now,
//     };

//     // Save to database
//     let db = get_adminx_database();
//     let collection = db.collection::<AdminxUser>("adminxs");
    
//     match collection.insert_one(&new_user, None).await {
//         Ok(result) => {
//             println!("✅ Admin user created successfully!");
//             println!("   ID: {}", result.inserted_id);
//             println!("   Email: {}", email);
//             println!("   Status: {:?}", new_user.status);
//         }
//         Err(e) => {
//             return Err(anyhow!("❌ Failed to create user: {}", e));
//         }
//     }

//     Ok(())
// }

// async fn list_users(matches: &ArgMatches) -> Result<()> {
//     let include_deleted = matches.get_flag("include-deleted");
//     let format = matches.get_one::<String>("format").unwrap();

//     let users = get_all_admins(include_deleted).await
//         .map_err(|e| anyhow!("Failed to fetch users: {}", e))?;

//     if users.is_empty() {
//         println!("No admin users found.");
//         return Ok(());
//     }

//     match format.as_str() {
//         "json" => {
//             let public_users: Vec<_> = users.iter().map(|u| u.to_public()).collect();
//             println!("{}", serde_json::to_string_pretty(&public_users)?);
//         }
//         "csv" => {
//             println!("ID,Username,Email,Status,Deleted,Created,Updated");
//             for user in users {
//                 println!(
//                     "{},{},{},{:?},{},{},{}",
//                     user.id.map(|id| id.to_string()).unwrap_or_default(),
//                     user.username,
//                     user.email,
//                     user.status,
//                     user.delete,
//                     user.created_at,
//                     user.updated_at
//                 );
//             }
//         }
//         "table" | _ => {
//             println!("\n📋 Admin Users:");
//             println!("{:-<100}", "");
//             println!(
//                 "{:<25} {:<20} {:<25} {:<12} {:<8}",
//                 "ID", "Username", "Email", "Status", "Deleted"
//             );
//             println!("{:-<100}", "");
            
//             for user in &users {
//                 let status_icon = match user.status {
//                     AdminxStatus::Active => "🟢",
//                     AdminxStatus::Inactive => "🟡", 
//                     AdminxStatus::Suspended => "🔴",
//                 };
                
//                 println!(
//                     "{:<25} {:<20} {:<25} {:<12} {:<8}",
//                     user.id.map(|id| id.to_string()).unwrap_or_default(),
//                     user.username,
//                     user.email,
//                     format!("{} {:?}", status_icon, user.status),
//                     if user.delete { "❌ Yes" } else { "✅ No" }
//                 );
//             }
//             println!("{:-<100}", "");
//             println!("Total: {} users\n", users.len());
//         }
//     }

//     Ok(())
// }

// async fn delete_user(matches: &ArgMatches) -> Result<()> {
//     let email = matches.get_one::<String>("email").unwrap();
//     let skip_confirmation = matches.get_flag("confirm");

//     // Find the user first
//     let user = get_admin_by_email(email).await
//         .ok_or_else(|| anyhow!("❌ User with email '{}' not found", email))?;

//     if user.delete {
//         return Err(anyhow!("❌ User '{}' is already deleted", email));
//     }

//     // Confirm deletion
//     if !skip_confirmation {
//         print!("⚠️  Are you sure you want to delete user '{}'? (y/N): ", email);
//         io::stdout().flush()?;
        
//         let mut input = String::new();
//         io::stdin().read_line(&mut input)?;
        
//         if !input.trim().to_lowercase().starts_with('y') {
//             println!("❌ Deletion cancelled.");
//             return Ok(());
//         }
//     }

//     // Perform soft delete
//     let user_id = user.id.ok_or_else(|| anyhow!("User has no ID"))?;
    
//     match delete_admin_by_id(&user_id).await {
//         Ok(true) => {
//             println!("✅ User '{}' has been soft deleted successfully.", email);
//         }
//         Ok(false) => {
//             return Err(anyhow!("❌ Failed to delete user '{}' - user not found", email));
//         }
//         Err(e) => {
//             return Err(anyhow!("❌ Database error while deleting user: {}", e));
//         }
//     }

//     Ok(())
// }

// async fn update_user_status(matches: &ArgMatches) -> Result<()> {
//     let email = matches.get_one::<String>("email").unwrap();
//     let status_str = matches.get_one::<String>("status").unwrap();

//     let new_status = match status_str.as_str() {
//         "active" => AdminxStatus::Active,
//         "inactive" => AdminxStatus::Inactive,
//         "suspended" => AdminxStatus::Suspended,
//         _ => return Err(anyhow!("Invalid status: {}", status_str)),
//     };

//     // Find the user first
//     let user = get_admin_by_email(email).await
//         .ok_or_else(|| anyhow!("❌ User with email '{}' not found", email))?;

//     let user_id = user.id.ok_or_else(|| anyhow!("User has no ID"))?;

//     match update_admin_status(&user_id, new_status.clone()).await {
//         Ok(true) => {
//             println!("✅ User '{}' status updated to '{:?}' successfully.", email, new_status);
//         }
//         Ok(false) => {
//             return Err(anyhow!("❌ Failed to update user '{}' status - user not found", email));
//         }
//         Err(e) => {
//             return Err(anyhow!("❌ Database error while updating user status: {}", e));
//         }
//     }

//     Ok(())
// }

// async fn show_user_info(matches: &ArgMatches) -> Result<()> {
//     let email = matches.get_one::<String>("email").unwrap();

//     let user = get_admin_by_email(email).await
//         .ok_or_else(|| anyhow!("❌ User with email '{}' not found", email))?;

//     println!("\n👤 User Information:");
//     println!("{:-<50}", "");
//     println!("ID:         {}", user.id.map(|id| id.to_string()).unwrap_or_default());
//     println!("Username:   {}", user.username);
//     println!("Email:      {}", user.email);
    
//     let status_display = match user.status {
//         AdminxStatus::Active => "🟢 Active",
//         AdminxStatus::Inactive => "🟡 Inactive",
//         AdminxStatus::Suspended => "🔴 Suspended",
//     };
//     println!("Status:     {}", status_display);
//     println!("Deleted:    {}", if user.delete { "❌ Yes" } else { "✅ No" });
//     println!("Created:    {}", user.created_at);
//     println!("Updated:    {}", user.updated_at);
//     println!("{:-<50}", "");

//     Ok(())
// }

// async fn show_db_stats() -> Result<()> {
//     let db = get_adminx_database();
    
//     // Get collection stats
//     let collection = db.collection::<AdminxUser>("adminxs");
    
//     let total_users = collection.count_documents(doc! {}, None).await?;
//     let active_users = collection.count_documents(doc! { "status": "active", "delete": false }, None).await?;
//     let deleted_users = collection.count_documents(doc! { "delete": true }, None).await?;
//     let suspended_users = collection.count_documents(doc! { "status": "suspended", "delete": false }, None).await?;

//     println!("\n📊 Database Statistics:");
//     println!("{:-<40}", "");
//     println!("Total Users:      {}", total_users);
//     println!("Active Users:     {}", active_users);
//     println!("Suspended Users:  {}", suspended_users);
//     println!("Deleted Users:    {}", deleted_users);
//     println!("{:-<40}", "");

//     Ok(())
// }

// // Helper functions for interactive input
// fn prompt_input(prompt: &str) -> Result<String> {
//     print!("{}: ", prompt);
//     io::stdout().flush()?;
    
//     let mut input = String::new();
//     io::stdin().read_line(&mut input)?;
    
//     Ok(input.trim().to_string())
// }

// fn prompt_password(prompt: &str) -> Result<String> {
//     print!("{}: ", prompt);
//     io::stdout().flush()?;
    
//     // For simplicity, using regular input. In production, you might want to use
//     // a crate like `rpassword` to hide password input
//     let mut input = String::new();
//     io::stdin().read_line(&mut input)?;
    
//     Ok(input.trim().to_string())
// }

// fn prompt_select(prompt: &str, options: &[&str], default: &str) -> Result<String> {
//     println!("{} (options: {}) [{}]:", prompt, options.join(", "), default);
//     print!("> ");
//     io::stdout().flush()?;
    
//     let mut input = String::new();
//     io::stdin().read_line(&mut input)?;
    
//     let choice = input.trim();
//     if choice.is_empty() {
//         Ok(default.to_string())
//     } else if options.contains(&choice) {
//         Ok(choice.to_string())
//     } else {
//         Err(anyhow!("Invalid option: {}. Must be one of: {}", choice, options.join(", ")))
//     }
// }