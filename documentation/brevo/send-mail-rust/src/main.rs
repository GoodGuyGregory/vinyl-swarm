use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use dotenv::dotenv;
use std::env;

fn main() {
    dotenv().ok();
    println!("Sending Emails with Rust!");

    // create variables for sending the email
    let smtp_key: String = env::var("SMTP_KEY").expect("SMTP_KEY must be set");
    let from_email: String = env::var("LOGIN").expect("LOGIN must be set");
    let smtp_server_host: String = env::var("SMTP_SERVER").expect("SMTP_SERVER must be set");
    let to_email: String = env::var("YOUR_EMAIL").expect("YOUR_EMAIL must be set");

    // create an email message to sendout to yourself
    let email: Message = Message::builder()
        .from(from_email.parse().unwrap())
        .to(to_email.parse().unwrap())
        .subject("Email SMTP Server Test")
        .body("This is a test email, let's hope it finds you well\n Thanks Rust".to_string())
        .unwrap();

    // build the transporter 
    let mailer: SmtpTransport = SmtpTransport::relay(&smtp_server_host)
        .unwrap()
        .credentials(Credentials::new(
            from_email,
            smtp_key
        ))
        .build();

    // build the SMTP credentials
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!\n Check {} Inbox", &to_email),
        Err(e) => println!("Couldn't send the email: {:?}", e),
    }

}
