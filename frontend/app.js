const SUPABASE_URL = "https://avdchsjlsuqnmdbxlrby.supabase.co";
const SUPABASE_ANON_KEY = "sb_publishable_HF_GNcqC04vKZ8T1fliN1A_tF0R7Eg1";

// Create the client using the global 'supabase' object from the CDN
const supabaseClient = supabase.createClient(SUPABASE_URL, SUPABASE_ANON_KEY);

// DOM Elements
const authView = document.getElementById('auth-view');
const dashboardView = document.getElementById('dashboard-view');
const emailInput = document.getElementById('email');
const passwordInput = document.getElementById('password');
const authSubmit = document.getElementById('auth-submit');
const toggleAuth = document.getElementById('toggle-auth');
const authStatus = document.getElementById('auth-status');

const uploadZone = document.getElementById('upload-zone');
const fileInput = document.getElementById('file-input');
const previewContainer = document.getElementById('preview-container');
const imagePreview = document.getElementById('image-preview');
const upscaleSubmit = document.getElementById('upscale-submit');
const status = document.getElementById('status');
const resultContainer = document.getElementById('result-container');
const resultImage = document.getElementById('result-image');
const downloadBtn = document.getElementById('download-btn');
const logoutBtn = document.getElementById('logout-btn');

let isSignUp = false;
let selectedFile = null;

// --- Auth Logic ---

toggleAuth.addEventListener('click', () => {
    isSignUp = !isSignUp;
    authSubmit.innerText = isSignUp ? "Create Account" : "Sign In";
    toggleAuth.innerHTML = isSignUp 
        ? "Already have an account? <span>Sign In</span>" 
        : "New here? <span>Create an account</span>";
});

authSubmit.addEventListener('click', async () => {
    const email = emailInput.value;
    const password = passwordInput.value;
    
    if (!email || !password) {
        showAuthError("Please enter both email and password.");
        return;
    }

    authSubmit.disabled = true;
    authSubmit.innerText = isSignUp ? "Creating..." : "Signing in...";
    authStatus.classList.add('hidden');

    try {
        const { data, error } = isSignUp 
            ? await supabaseClient.auth.signUp({ email, password })
            : await supabaseClient.auth.signInWithPassword({ email, password });

        if (error) throw error;
        
        if (isSignUp && data.user && data.session === null) {
            showAuthError("Please check your email for a confirmation link!", "success-message");
        } else {
            await checkUser();
        }
    } catch (err) {
        showAuthError(err.message);
    } finally {
        authSubmit.disabled = false;
        authSubmit.innerText = isSignUp ? "Create Account" : "Sign In";
    }
});

function showAuthError(msg, className = "error-message") {
    authStatus.innerText = msg;
    authStatus.className = `status-message ${className}`;
    authStatus.classList.remove('hidden');
}

async function checkUser() {
    const { data: { session } } = await supabaseClient.auth.getSession();
    if (session) {
        authView.classList.add('hidden');
        dashboardView.classList.remove('hidden');
    } else {
        authView.classList.remove('hidden');
        dashboardView.classList.add('hidden');
    }
}

logoutBtn.addEventListener('click', async () => {
    await supabaseClient.auth.signOut();
    await checkUser();
});

// --- Upload Logic ---

uploadZone.addEventListener('click', () => fileInput.click());

uploadZone.addEventListener('dragover', (e) => {
    e.preventDefault();
    uploadZone.classList.add('dragover');
});

uploadZone.addEventListener('dragleave', () => {
    uploadZone.classList.remove('dragover');
});

uploadZone.addEventListener('drop', (e) => {
    e.preventDefault();
    uploadZone.classList.remove('dragover');
    if (e.dataTransfer.files && e.dataTransfer.files[0]) {
        handleFile(e.dataTransfer.files[0]);
    }
});

fileInput.addEventListener('change', (e) => {
    if (e.target.files && e.target.files[0]) {
        handleFile(e.target.files[0]);
    }
});

function handleFile(file) {
    if (!file) return;

    // 1. Client-Side Size Check
    const MAX_SIZE = 15 * 1024 * 1024; // 15MB
    if (file.size > MAX_SIZE) {
        showStatus("Error: File exceeds 15MB limit.", "error-message");
        return;
    }

    selectedFile = file;
    const reader = new FileReader();
    reader.onload = (e) => {
        imagePreview.src = e.target.result;
        previewContainer.classList.remove('hidden');
        uploadZone.classList.add('hidden');
        resultContainer.classList.add('hidden');
        showStatus("");
    };
    reader.readAsDataURL(file);
}

// --- API Logic ---

upscaleSubmit.addEventListener('click', async () => {
    if (!selectedFile) return;

    // 2. Lock UI
    upscaleSubmit.disabled = true;
    upscaleSubmit.innerHTML = `<span class="loading-spinner"></span> Processing AI Upscale...`;
    showStatus("Sending to Gemini API...", "success-message");

    const formData = new FormData();
    formData.append("image", selectedFile);

    const { data: { session } } = await supabaseClient.auth.getSession();
    if (!session) {
        showStatus("Session expired. Please log in again.", "error-message");
        checkUser();
        return;
    }
    
    const token = session.access_token;

    try {
        const response = await fetch('/upscale', {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${token}`
            },
            body: formData
        });

        if (!response.ok) {
            const errText = await response.text();
            throw new Error(errText || "Upscale failed");
        }

        const data = await response.json();
        
        // Success
        resultImage.src = data.image_url;
        resultContainer.classList.remove('hidden');
        previewContainer.classList.add('hidden');
        showStatus("Optimization Complete!", "success-message");

    } catch (err) {
        showStatus(`Error: ${err.message}`, "error-message");
    } finally {
        upscaleSubmit.disabled = false;
        upscaleSubmit.innerText = "Start Upscale";
    }
});

downloadBtn.addEventListener('click', () => {
    const link = document.createElement('a');
    link.href = resultImage.src;
    link.download = `upscaled-${Date.now()}.png`;
    link.click();
});

function showStatus(msg, className = "") {
    status.innerText = msg;
    status.className = "status-message " + className;
}

// Initial Session Check
checkUser();
