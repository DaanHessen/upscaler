# Privacy Policy

**Last Updated:** April 22, 2026
**Version:** v1.0.0 (Production)

UPSYL STUDIO ("we," "us," or "our") is committed to protecting your privacy. This policy explains how we collect, use, and share your information when you use our AI upscaling services.

## 1. Information We Collect
*   **Account Identifiers**: We collect your email address and authentication metadata via our identity provider, **Supabase**.
*   **Image Content**: We temporarily store the images you upload and the generated results.
*   **Payment Data**: All payments are processed by **Stripe**. We store your Stripe Customer ID and transaction history, but we never store or process your full credit card details.
*   **System Telemetry**: We store technical logs including processing latency, token usage (via Google Vertex AI), and resolution metadata (e.g., 2K vs 4K).

## 2. Third-Party Data Processors
To provide our service, we share specific data with these trusted providers:
*   **Google Cloud (Vertex AI)**: Images are transmitted to Google for processing using Gemini AI. Google's Cloud Terms apply, ensuring your data is not used for training their foundation models unless opted-in.
*   **Stripe**: Used for secure payment processing.
*   **AWS / Google Cloud Storage**: Used for encrypted storage of your temporary image files.

## 3. Data Retention: The "24-Hour" Rule
We implement a strict data minimization policy. 
*   **Image Data**: All uploaded and upscaled images are **physically deleted** from our storage servers automatically **24 hours** after the job is completed.
*   **Moderation Logs**: Images flagged for safety violations (NSFW) may be retained for up to 30 days in a secure, audited environment for legal compliance and security review before deletion.
*   **Account Data**: Your email and credit balance are stored as long as your account is active.

## 4. Your Rights (GDPR & CCPA)
Under the GDPR (EU) and CCPA (California), you have the right to:
*   **Access**: Request a copy of the data we hold about you.
*   **Deletion**: Request that we delete your account and all associated data.
*   **Rectification**: Correct inaccurate personal data.
*   **Opt-Out**: Withdraw consent for data processing at any time (though this may result in service termination).

## 5. Security Measures
*   **Signed URLs**: All image downloads are protected by temporary, expiring signed URLs.
*   **JWT Auth**: API access is secured using industry-standard JSON Web Tokens.
*   **Encryption**: Data is encrypted at rest and in transit.

## 6. Contact Information
For privacy requests or to exercise your data rights, contact: **support@upsyl.studio**
