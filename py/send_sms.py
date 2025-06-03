import smtplib
from email.mime.text import MIMEText


def send_sms(phone_number, carrier_gateway, message, gmail_user, gmail_app_password):
    # Create the SMS email address
    sms_email = phone_number + carrier_gateway

    # Create email message
    msg = MIMEText(message)
    msg["Subject"] = ""  # Keep empty for cleaner SMS
    msg["From"] = gmail_user
    msg["To"] = sms_email

    # Send via Gmail SMTP
    try:
        with smtplib.SMTP("smtp.gmail.com", 587) as server:
            server.starttls()
            server.login(gmail_user, gmail_app_password)
            server.send_message(msg)
        print("SMS sent successfully!")
    except Exception as e:
        print(f"Error: {e}")


if __name__ == "__main__":
    # Usage example
    send_sms(
        phone_number="5307509190",  # Your phone number (no dashes/spaces)
        carrier_gateway="@txt.att.net",  # Your carrier's gateway
        message="Hello from Python!",
        gmail_user="guilhermedemouraa@gmail.com",  # Your Gmail address
        gmail_app_password="schy jmik kfja gryg",  # The 16-character app password
    )
