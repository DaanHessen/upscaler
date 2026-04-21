# Privacy Policy

**Last Updated:** April 21, 2026
**Version:** v1.0.0

Upsyl ("we," "us," or "our") is committed to protecting your privacy. This policy explains how we collect, use, and share your information when you use our AI upscaling service.

## 1. Information We Collect
- **Account Information**: When you sign up, we collect your email address and authentication identifiers via Supabase.
- **Image Data**: We process the images you upload to provide our upscaling service.
- **Payment Information**: Payment processing is handled by Stripe. We do not store your full credit card details; we store transaction identifiers provided by Stripe.
- **Usage Metadata**: We store logs of your upscaling history, including status, resolution (2K/4K), and processing timestamps.

## 2. How We Use Your Information
- To provide and maintain our upscaling service.
- To manage your credit balance and process payments.
- To improve our service through automated style analysis and moderation.
- To comply with legal obligations and our content guidelines.

## 3. Third-Party Service Providers
We share certain data with trusted third parties to function:
- **Supabase**: Authentication and Database services.
- **Google Cloud (Vertex AI)**: Image processing and enhancement using the Gemini 1.5 Pro model.
- **Stripe**: Payment processing and billing.
- **Google Cloud Storage (GCS)**: Secure storage of uploaded and processed images.

## 4. Data Retention and Deletion
- **Images**: Uploaded and generated images are automatically deleted from our production storage after **24 hours**.
- **Moderation Logs**: Images rejected for NSFW violations may be stored in a separate, restricted folder for up to 30 days for audit and security purposes before deletion.
- **Accounts**: You may request account deletion at any time, which will remove your email and usage history from our records.

## 5. Security
We implement industry-standard security measures, including:
- **JWT Authentication**: Secured access via Supabase tokens.
- **Signed URLs**: Temporary, expiring links for image downloads.
- **Rate Limiting**: Protection against abuse and automated scraping.

## 6. Your Rights (GDPR/CCPA)
Depending on your location, you have rights regarding your personal data, including:
- The right to access your data.
- The right to rectification or deletion.
- The right to withdraw consent for processing.

---
*For privacy inquiries, please contact us.*
