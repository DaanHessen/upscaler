/* =========================================================
   Gemini Upscaler — Application Logic v2
   SaaS Platform with Sidebar Navigation & Premium Features
   ========================================================= */

// ===========================
//  1. Configuration & Constants
// ===========================

const SUPABASE_URL  = "https://avdchsjlsuqnmdbxlrby.supabase.co";
const SUPABASE_ANON = "sb_publishable_HF_GNcqC04vKZ8T1fliN1A_tF0R7Eg1";

const COST_TABLE    = { "1K": 1, "2K": 2, "4K": 4 };
const MAX_FILE_SIZE = 15 * 1024 * 1024; // 15 MB
const POLL_INTERVAL = 3000;             // ms

// ===========================
//  2. State Management
// ===========================

let selectedFile      = null;      // Current image file
let detectedStyle     = null;      // "PHOTOGRAPHY" | "ILLUSTRATION"
let selectedQuality   = "2K";      // Current quality selection
let selectedStyle     = "AUTO";    // Current style selection
let temperature       = 0.0;       // Gemini creativity
let currentJobId      = null;      // Active processing job ID
let creditBalance     = null;      // User credit balance
let pollTimer         = null;      // Polling handle
let historyData       = [];        // Cached history records
let activePage        = "upscale"; // "upscale" | "history" | "billing" | "account"

const sb = supabase.createClient(SUPABASE_URL, SUPABASE_ANON);

// ===========================
//  3. DOM References
// ===========================

const $ = (id) => document.getElementById(id);

const dom = {
    // Top Level Shells
    authOverlay:   $("auth-overlay"),
    appShell:      $("app-shell"),
    
    // Auth
    authForm:      $("auth-form"),
    authEmail:     $("auth-email"),
    authPassword:  $("auth-password"),
    authBtn:       $("auth-btn"),
    authToggleLink:$("auth-toggle-link"),
    authToggleText:$("auth-toggle-text"),
    authMsg:       $("auth-msg"),
    
    // Navigation
    sidebar:       $("sidebar"),
    sidebarNav:    document.querySelectorAll(".nav-item"),
    sidebarLogout: $("sidebar-logout"),
    sidebarEmail:  $("sidebar-email"),
    sidebarOverlay:$("sidebar-overlay"),
    hamburger:     $("hamburger"),
    topbarTitle:   $("topbar-title"),
    userChip:      $("user-chip"),
    
    // Global Elements
    creditCount:   $("credit-count"),
    creditBadge:   $("credit-badge"),
    toastWrap:     $("toast-wrap"),
    
    // Page: Upscale (Workspace States)
    pages:         document.querySelectorAll(".page"),
    stUpload:      $("st-upload"),
    stModerating:  $("st-moderating"),
    stConfig:      $("st-config"),
    stProcessing:  $("st-processing"),
    stResult:      $("st-result"),
    
    // Workspace Elements
    dropzone:      $("dropzone"),
    fileInput:     $("file-input"),
    imgPreview:    $("img-preview"),
    detectedLabel: $("detected-label"),
    pillsStyle:    $("pills-style"),
    pillsQuality:  $("pills-quality"),
    tempSlider:    $("temp-slider"),
    tempVal:       $("temp-val"),
    costNum:       $("cost-num"),
    btnUpscale:    $("btn-upscale"),
    btnBack:       $("btn-back"),
    procStatus:    $("proc-status"),
    
    // Result Slider
    compareBox:    $("compare-box"),
    compareBefore: $("compare-before"),
    compareAfter:  $("compare-after"),
    btnDownload:   $("btn-download"),
    btnNew:        $("btn-new"),
    resultMeta:    $("result-meta"),
    
    // Page: History
    historyGrid:   $("history-grid"),
    historyEmpty:  $("history-empty"),
    filterBar:     $("filter-bar"),
    
    // Page: Billing
    billingBalance:$("billing-balance"),
    buyBtns:       document.querySelectorAll(".buy-btn"),
    
    // Page: Account
    acctAvatar:    $("acct-avatar"),
    acctEmail:     $("acct-email"),
    acctSince:     $("acct-since"),
    acctTotal:     $("acct-total"),
    acctCompleted: $("acct-completed"),
    acctBalance:   $("acct-balance"),
    pwForm:        $("pw-form"),
};

// ===========================
//  4. Initialization
// ===========================

document.addEventListener("DOMContentLoaded", () => {
    initIcons();
    initNavigation();
    initAuth();
    initUpload();
    initWorkspace();
    initSlider();
    initBilling();
    initAccount();
    checkPaymentParams();
    checkSession();
});

function initIcons() {
    if (window.lucide) lucide.createIcons();
}

// ===========================
//  5. Navigation Logic
// ===========================

function initNavigation() {
    // Tab switching
    dom.sidebarNav.forEach(btn => {
        btn.addEventListener("click", () => {
            const pageId = btn.dataset.page;
            switchToPage(pageId);
        });
    });

    // Mobile Hamburger
    dom.hamburger.addEventListener("click", () => {
        document.body.classList.add("sidebar-open");
    });

    dom.sidebarOverlay.addEventListener("click", () => {
        document.body.classList.remove("sidebar-open");
    });
}

function switchToPage(pageId) {
    activePage = pageId;
    
    // Update Sidebar UI
    dom.sidebarNav.forEach(btn => {
        const isActive = btn.dataset.page === pageId;
        btn.classList.toggle("active", isActive);
    });

    // Update Content Visibility
    dom.pages.forEach(p => {
        const isTarget = p.id === `page-${pageId}`;
        p.classList.toggle("active", isTarget);
    });

    // Update Title
    const titleMap = {
        upscale: "Upscale",
        history: "History",
        billing: "Billing",
        account: "Account"
    };
    dom.topbarTitle.textContent = titleMap[pageId] || "Dashboard";

    // Trigger page-specific loads
    if (pageId === "history") loadHistory();
    if (pageId === "billing") loadBalance();
    if (pageId === "account") syncAccountDetails();

    // Close mobile menu
    document.body.classList.remove("sidebar-open");
    initIcons();
}

// ===========================
//  6. Authentication Module
// ===========================

let isSignUpMode = false;
let isSessionChecking = false;

function initAuth() {
    dom.authForm.addEventListener("submit", handleAuthSubmit);
    
    dom.authToggleLink.addEventListener("click", (e) => {
        e.preventDefault();
        isSignUpMode = !isSignUpMode;
        
        dom.authBtn.textContent = isSignUpMode ? "Create Account" : "Sign In";
        dom.authToggleText.textContent = isSignUpMode ? "Already have an account?" : "New here?";
        dom.authToggleLink.textContent = isSignUpMode ? "Sign in" : "Create an account";
        hideAuthMsg();
    });

    dom.sidebarLogout.addEventListener("click", async () => {
        await sb.auth.signOut();
        checkSession();
        showToast("Logged out successfully.", "info");
    });
}

async function handleAuthSubmit(e) {
    e.preventDefault();
    const email = dom.authEmail.value.trim();
    const password = dom.authPassword.value;

    if (!email || !password) {
        showAuthMsg("Double check your credentials.", "err");
        return;
    }

    setAuthLoading(true);
    hideAuthMsg();

    try {
        const { data, error } = isSignUpMode
            ? await sb.auth.signUp({ email, password })
            : await sb.auth.signInWithPassword({ email, password });

        if (error) throw error;

        if (isSignUpMode && data.user && !data.session) {
            showAuthMsg("Success! Check your email for a join link.", "ok");
        } else {
            await checkSession();
        }
    } catch (err) {
        showAuthMsg(err.message, "err");
    } finally {
        setAuthLoading(false);
    }
}

function setAuthLoading(isLoading) {
    dom.authBtn.disabled = isLoading;
    dom.authBtn.innerHTML = isLoading 
        ? `<span class="btn-spinner"></span> ${isSignUpMode ? "Creating..." : "Signing in..."}`
        : (isSignUpMode ? "Create Account" : "Sign In");
}

function showAuthMsg(text, type) {
    dom.authMsg.textContent = text;
    dom.authMsg.className = `auth-msg ${type}`;
    dom.authMsg.classList.remove("hidden");
}

function hideAuthMsg() {
    dom.authMsg.classList.add("hidden");
}

async function checkSession() {
    if (isSessionChecking) return;
    isSessionChecking = true;

    try {
        const { data: { session } } = await sb.auth.getSession();
        
        if (session) {
            dom.authOverlay.classList.add("hidden");
            dom.appShell.classList.remove("hidden");
            
            const email = session.user.email;
            dom.sidebarEmail.textContent = email;
            dom.userChip.textContent = email.charAt(0).toUpperCase();
            dom.acctEmail.textContent = email;
            dom.acctAvatar.textContent = email.charAt(0).toUpperCase();
            dom.acctSince.textContent = "Joined " + new Date(session.user.created_at).toLocaleDateString();
            
            // Initial data load
            loadBalance();
            syncStats();
        } else {
            dom.authOverlay.classList.remove("hidden");
            dom.appShell.classList.add("hidden");
        }
    } catch (err) {
        console.error("Session check error:", err);
    } finally {
        isSessionChecking = false;
        initIcons();
    }
}

async function getAuthToken() {
    const { data: { session } } = await sb.auth.getSession();
    return session ? session.access_token : null;
}

// ===========================
//  7. Balance & Billing
// ===========================

async function loadBalance() {
    const token = await getAuthToken();
    if (!token) return;

    try {
        const res = await fetch("/balance", {
            headers: { "Authorization": `Bearer ${token}` }
        });
        if (!res.ok) throw new Error("Balance sync failed");
        
        const data = await res.json();
        updateBalanceUI(data.credits);
    } catch (err) {
        console.error(err);
    }
}

function updateBalanceUI(val) {
    const prev = creditBalance;
    creditBalance = val;

    // Update all balance displays
    const elements = [dom.creditCount, $("stat-credits"), dom.billingBalance, dom.acctBalance];
    
    elements.forEach(el => {
        if (!el) return;
        if (prev !== null && prev !== val && el === dom.creditCount) {
            animateNumber(el, prev, val, 600);
            dom.creditBadge.classList.add("pulse");
            setTimeout(() => dom.creditBadge.classList.remove("pulse"), 600);
        } else {
            el.textContent = val;
        }
    });
}

function initBilling() {
    dom.buyBtns.forEach(btn => {
        btn.addEventListener("click", () => startCheckout(btn.dataset.tier));
    });
}

async function startCheckout(tier) {
    const token = await getAuthToken();
    if (!token) return;

    const btn = document.querySelector(`.buy-btn[data-tier="${tier}"]`);
    btn.disabled = true;
    btn.innerHTML = '<span class="btn-spinner"></span>';

    try {
        const res = await fetch("/checkout", {
            method: "POST",
            headers: {
                "Authorization": `Bearer ${token}`,
                "Content-Type": "application/json"
            },
            body: JSON.stringify({ tier })
        });
        
        const data = await res.json();
        if (data.url) window.location.href = data.url;
        else throw new Error(data.error || "Checkout error");
        
    } catch (err) {
        showToast(err.message, "error");
        btn.disabled = false;
        btn.textContent = "Purchase";
    }
}

// ===========================
//  8. Workspace State Logic
// ===========================

const STATES = ["stUpload", "stModerating", "stConfig", "stProcessing", "stResult"];

function transitionTo(stateName) {
    STATES.forEach(s => {
        dom[s].classList.toggle("hidden", s !== stateName);
    });
    initIcons();
}

function initUpload() {
    dom.dropzone.addEventListener("click", () => dom.fileInput.click());
    
    dom.dropzone.addEventListener("dragover", (e) => {
        e.preventDefault();
        dom.dropzone.classList.add("dragover");
    });
    
    dom.dropzone.addEventListener("dragleave", () => dom.dropzone.classList.remove("dragover"));
    
    dom.dropzone.addEventListener("drop", (e) => {
        e.preventDefault();
        dom.dropzone.classList.remove("dragover");
        if (e.dataTransfer.files?.[0]) handleFile(e.dataTransfer.files[0]);
    });

    dom.fileInput.addEventListener("change", (e) => {
        if (e.target.files?.[0]) handleFile(e.target.files[0]);
    });
}

function handleFile(file) {
    if (!file.type.startsWith("image/")) {
        showToast("Invalid file type.", "warning");
        return;
    }
    if (file.size > MAX_FILE_SIZE) {
        showToast("File too large (>15MB).", "warning");
        return;
    }

    selectedFile = file;
    
    // Preview original
    const reader = new FileReader();
    reader.onload = (e) => {
        dom.imgPreview.src = e.target.result;
        dom.compareBefore.src = e.target.result;
    };
    reader.readAsDataURL(file);

    startModeration(file);
}

async function startModeration(file) {
    transitionTo("stModerating");
    
    const token = await getAuthToken();
    if (!token) { transitionTo("stUpload"); return; }

    const fd = new FormData();
    fd.append("image", file);

    try {
        const res = await fetch("/moderate", {
            method: "POST",
            headers: { "Authorization": `Bearer ${token}` },
            body: fd
        });

        if (!res.ok) throw new Error("Network error during analysis");
        
        const data = await res.json();
        if (data.nsfw) {
            showToast("Safety Check: Image rejected.", "error");
            resetUpscaleFlow();
            return;
        }

        detectedStyle = data.detected_style;
        dom.detectedLabel.textContent = detectedStyle === "ILLUSTRATION" ? "Illustration" : "Photography";
        
        // Reset config view
        resetConfigSliders();
        transitionTo("stConfig");

    } catch (err) {
        showToast(err.message, "error");
        transitionTo("stUpload");
    }
}

function initWorkspace() {
    // Quality Pills
    dom.pillsQuality.addEventListener("click", (e) => {
        const p = e.target.closest(".pill");
        if (!p) return;
        selectedQuality = p.dataset.v;
        updatePills(dom.pillsQuality, selectedQuality);
        updateCost();
    });

    // Style Pills
    dom.pillsStyle.addEventListener("click", (e) => {
        const p = e.target.closest(".pill");
        if (!p) return;
        selectedStyle = p.dataset.v;
        updatePills(dom.pillsStyle, selectedStyle);
    });

    // Slider
    dom.tempSlider.addEventListener("input", () => {
        temperature = parseFloat(dom.tempSlider.value);
        dom.tempVal.textContent = temperature.toFixed(1);
        const pct = (temperature / 2) * 100;
        dom.tempSlider.style.setProperty("--progress", `${pct}%`);
    });

    dom.btnBack.addEventListener("click", resetUpscaleFlow);
    dom.btnUpscale.addEventListener("click", startUpscale);
    
    dom.btnNew.addEventListener("click", resetUpscaleFlow);
    dom.btnDownload.addEventListener("click", () => {
        const link = document.createElement("a");
        link.href = dom.compareAfter.src;
        link.download = `upscaled_${Date.now()}.png`;
        link.click();
    });
}

function updatePills(container, val) {
    container.querySelectorAll(".pill").forEach(p => {
        p.classList.toggle("active", p.dataset.v === val);
    });
}

function updateCost() {
    dom.costNum.textContent = COST_TABLE[selectedQuality] || 0;
}

function resetConfigSliders() {
    selectedQuality = "2K";
    selectedStyle = "AUTO";
    temperature = 0.0;
    updatePills(dom.pillsQuality, "2K");
    updatePills(dom.pillsStyle, "AUTO");
    dom.tempSlider.value = 0;
    dom.tempSlider.style.setProperty("--progress", "0%");
    dom.tempVal.textContent = "0.0";
    updateCost();
}

function resetUpscaleFlow() {
    selectedFile = null;
    currentJobId = null;
    detectedStyle = null;
    dom.fileInput.value = "";
    if (pollTimer) clearTimeout(pollTimer);
    transitionTo("stUpload");
}

async function startUpscale() {
    const cost = COST_TABLE[selectedQuality];
    if (creditBalance !== null && creditBalance < cost) {
        showToast(`Need ${cost} credits. You have ${creditBalance}.`, "warning");
        switchToPage("billing");
        return;
    }

    const token = await getAuthToken();
    if (!token) return;

    dom.btnUpscale.disabled = true;
    dom.btnUpscale.innerHTML = '<span class="btn-spinner"></span> Polishing...';

    const fd = new FormData();
    fd.append("image", selectedFile);
    fd.append("quality", selectedQuality);
    fd.append("temperature", temperature);
    if (selectedStyle !== "AUTO") fd.append("style", selectedStyle);

    try {
        const res = await fetch("/upscale", {
            method: "POST",
            headers: { "Authorization": `Bearer ${token}` },
            body: fd
        });

        if (res.status === 402) throw new Error("Insufficient credits");
        if (!res.ok) throw new Error("Upload failed");

        const data = await res.json();
        currentJobId = data.job_id;
        
        transitionTo("stProcessing");
        dom.procStatus.textContent = "Processing...";
        pollJob(currentJobId, token);

    } catch (err) {
        showToast(err.message, "error");
        dom.btnUpscale.disabled = false;
        dom.btnUpscale.innerHTML = '<i data-lucide="zap"></i> Upscale Now';
        initIcons();
    }
}

async function pollJob(id, token) {
    try {
        const res = await fetch(`/upscales/${id}`, {
            headers: { "Authorization": `Bearer ${token}` }
        });
        
        if (res.status === 401 || res.status === 403) {
            showToast("Session expired — please sign in again.", "warning");
            resetUpscaleFlow();
            checkSession();
            return;
        }

        if (res.status === 404) {
            showToast("Upscale job not found.", "error");
            resetUpscaleFlow();
            return;
        }

        if (!res.ok) throw new Error();
        
        const data = await res.json();

        if (data.status === "COMPLETED") {
            showResult(data);
            return;
        }
        if (data.status === "FAILED") {
            showToast("Processing error: " + (data.error || "Gemini timed out"), "error");
            resetUpscaleFlow();
            return;
        }

        // Animated status
        const dots = ".".repeat((Date.now() / 500) % 4);
        dom.procStatus.textContent = "Enhancing with Gemini AI" + dots;

        pollTimer = setTimeout(() => pollJob(id, token), POLL_INTERVAL);
    } catch (err) {
        pollTimer = setTimeout(() => pollJob(id, token), POLL_INTERVAL);
    }
}

function showResult(data) {
    dom.compareAfter.src = data.image_url;
    
    // Meta pill rendering
    dom.resultMeta.innerHTML = `
        <span class="meta-pill">${data.quality}</span>
        <span class="meta-pill">${data.style || "Auto"}</span>
        <span class="meta-pill">T=${data.temperature}</span>
    `;
    
    transitionTo("stResult");
    showToast("Upscale successful!", "success");
    loadBalance();
    syncStats();
}

// ===========================
//  9. Comparison Slider
// ===========================

function initSlider() {
    const box = dom.compareBox;
    let isDragging = false;

    const move = (e) => {
        if (!isDragging) return;
        const rect = box.getBoundingClientRect();
        const x = (e.pageX || e.touches[0].pageX) - rect.left;
        let pct = (x / rect.width) * 100;
        pct = Math.max(0, Math.min(100, pct));
        box.style.setProperty("--split", pct + "%");
    };

    box.addEventListener("mousedown", () => isDragging = true);
    window.addEventListener("mouseup", () => isDragging = false);
    window.addEventListener("mousemove", move);
    
    box.addEventListener("touchstart", () => isDragging = true);
    window.addEventListener("touchend", () => isDragging = false);
    box.addEventListener("touchmove", move);
}

// ===========================
//  10. History Module
// ===========================

async function loadHistory() {
    const token = await getAuthToken();
    if (!token) return;

    try {
        const res = await fetch("/history", {
            headers: { "Authorization": `Bearer ${token}` }
        });
        const records = await res.json();
        renderHistory(records);
    } catch (err) {
        console.error(err);
    }
}

function renderHistory(items) {
    dom.historyGrid.querySelectorAll(".history-card").forEach(c => c.remove());
    
    if (!items || items.length === 0) {
        dom.historyEmpty.classList.remove("hidden");
        dom.historyEmpty.innerHTML = `
            <div class="empty-state">
                <div class="empty-icon-ring">
                    <i data-lucide="image"></i>
                </div>
                <h3>No upscales yet</h3>
                <p>Your processed images will appear here.</p>
                <button class="btn btn-primary btn-sm" onclick="switchToPage('upscale')" style="margin-top: 1rem;">
                    <i data-lucide="plus"></i> Start Upscaling
                </button>
            </div>
        `;
        initIcons();
        return;
    }
    
    dom.historyEmpty.classList.add("hidden");
    items.forEach((item, idx) => {
        const card = document.createElement("div");
        card.className = "history-card shadow-sm";
        card.style.animationDelay = (idx * 50) + "ms";

        const statusLabel = item.status.charAt(0) + item.status.slice(1).toLowerCase();
        const dateStr = new Date(item.created_at).toLocaleDateString([], { month: "short", day: "numeric", year: "numeric" });
        
        card.innerHTML = `
            <div class="hist-media">
                ${item.image_url ? 
                    `<img src="${item.image_url}" class="hist-thumb" alt="Result" loading="lazy">` : 
                    `<div class="hist-thumb-placeholder"><i data-lucide="image"></i></div>`
                }
                <div class="hist-status-overlay">
                    <span class="status-tag ${item.status.toLowerCase()}">${statusLabel}</span>
                </div>
            </div>
            <div class="hist-body">
                <div class="hist-meta-row">
                    <span class="hist-date">${dateStr}</span>
                    <div class="hist-config-pills">
                        <span class="meta-pill">${item.quality}</span>
                        <span class="meta-pill">${item.style || "Auto"}</span>
                    </div>
                </div>
                ${item.status === "COMPLETED" ? 
                    `<button class="hist-view-btn" data-url="${item.image_url}">
                        <i data-lucide="eye"></i> View Result
                    </button>` : 
                    item.status === "FAILED" ? 
                    `<div class="hist-error-box"><i data-lucide="alert-circle"></i> Error</div>` :
                    `<div class="hist-pulse-box"><span class="btn-spinner-sm"></span> Processing</div>`
                }
            </div>
        `;
        dom.historyGrid.appendChild(card);
    });

    dom.historyGrid.querySelectorAll(".hist-view-btn").forEach(btn => {
        btn.addEventListener("click", () => {
            if (btn.dataset.url && btn.dataset.url !== "undefined") {
                window.open(btn.dataset.url, "_blank");
            } else {
                showToast("Image URL is not available yet.", "warning");
            }
        });
    });
    
    initIcons();
}

// ===========================
//  11. Account & Stats
// ===========================

function initAccount() {
    dom.pwForm.addEventListener("submit", async (e) => {
        e.preventDefault();
        const pw = $("new-pw").value;
        const confirm = $("confirm-pw").value;
        
        if (pw !== confirm) {
            showToast("Passwords do not match", "error");
            return;
        }

        const { error } = await sb.auth.updateUser({ password: pw });
        if (error) showToast(error.message, "error");
        else {
            showToast("Password updated!", "success");
            dom.pwForm.reset();
        }
    });

    // Preferences sync
    $("pref-quality").addEventListener("click", (e) => {
        const p = e.target.closest(".pill");
        if (p) updatePills($("pref-quality"), p.dataset.v);
    });
    $("pref-style").addEventListener("click", (e) => {
        const p = e.target.closest(".pill");
        if (p) updatePills($("pref-style"), p.dataset.v);
    });
}

async function syncStats() {
    const token = await getAuthToken();
    if (!token) return;

    try {
        const res = await fetch("/history", {
            headers: { "Authorization": `Bearer ${token}` }
        });
        const items = await res.json();
        
        const total = items.length;
        const completed = items.filter(i => i.status === "COMPLETED").length;
        
        if ($("stat-upscales")) $("stat-upscales").textContent = total;
        if (dom.acctTotal) dom.acctTotal.textContent = total;
        if (dom.acctCompleted) dom.acctCompleted.textContent = completed;
    } catch (err) {}
}

function syncAccountDetails() {
    if (creditBalance !== null) dom.acctBalance.textContent = creditBalance;
    syncStats();
}

// ===========================
//  12. Helpers & Toasts
// ===========================

function showToast(msg, type = "info") {
    const t = document.createElement("div");
    t.className = `toast t-${type}`;
    
    const icons = { success: "check-circle", error: "alert-circle", warning: "alert-triangle", info: "info" };
    t.innerHTML = `<i data-lucide="${icons[type] || "info"}"></i><span>${msg}</span>`;
    
    dom.toastWrap.appendChild(t);
    initIcons();
    
    const kill = () => {
        t.classList.add("exit");
        setTimeout(() => t.remove(), 300);
    };
    
    setTimeout(kill, 4000);
    t.addEventListener("click", kill);
}

function checkPaymentParams() {
    const p = new URLSearchParams(window.location.search);
    if (p.get("payment") === "success") {
        showToast("Credits added! Ready to upscale.", "success");
        window.history.replaceState({}, "", "/");
    }
}

function animateNumber(el, from, to, duration) {
    let start = null;
    const step = (now) => {
        if (!start) start = now;
        const prog = Math.min((now - start) / duration, 1);
        el.textContent = Math.floor(from + (to - from) * prog);
        if (prog < 1) requestAnimationFrame(step);
    };
    requestAnimationFrame(step);
}
