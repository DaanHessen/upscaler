/* =========================================================
   Upscaler — Landing Page Logic
   Handles Hero Slider, Scroll Animations, and Auth Navigation
   ========================================================= */

document.addEventListener("DOMContentLoaded", () => {
    initHeroSlider();
    initLandingNav();
    initScrollAnimations();
});

function initHeroSlider() {
    const box = document.getElementById("hero-compare");
    if (!box) return;

    const move = (e) => {
        const rect = box.getBoundingClientRect();
        const x = (e.pageX || (e.touches && e.touches[0].pageX)) - rect.left;
        let pct = (x / rect.width) * 100;
        pct = Math.max(0, Math.min(100, pct));
        box.style.setProperty("--split", pct + "%");
    };

    // Landing hero slider should work on mouse move (hover feel)
    box.addEventListener("mousemove", move);
    box.addEventListener("touchmove", move);
}

function initLandingNav() {
    const getStartedBtn = document.getElementById("get-started-btn");
    const authSection = document.getElementById("auth-section");

    if (getStartedBtn && authSection) {
        getStartedBtn.addEventListener("click", () => {
            authSection.scrollIntoView({ behavior: "smooth" });
            // Add a subtle highlight effect to the auth card
            const card = authSection.querySelector(".auth-card");
            if (card) {
                gsap.fromTo(card, { scale: 1 }, { scale: 1.02, duration: 0.2, yoyo: true, repeat: 1 });
            }
        });
    }
}

function initScrollAnimations() {
    // Basic reveal logic for features
    const cards = document.querySelectorAll(".feature-card");
    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.classList.add("visible");
            }
        });
    }, { threshold: 0.1 });

    cards.forEach(card => observer.observe(card));
}
