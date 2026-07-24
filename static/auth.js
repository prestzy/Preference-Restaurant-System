/**
 * Adds submission feedback without replacing the browser's normal form POST.
 *
 * Registration and admin login intentionally remain server-rendered forms, so
 * they still work when JavaScript is unavailable or restricted by a preview
 * browser. This handler only prevents accidental double submission.
 */
document.addEventListener("DOMContentLoaded", () => {
  document.querySelectorAll("[data-auth-form]").forEach((form) => {
    form.addEventListener("submit", () => {
      const button = form.querySelector('button[type="submit"]');
      if (!button || button.disabled) return;

      button.dataset.originalLabel = button.textContent;
      button.textContent = button.dataset.submittingLabel || "Submitting...";
      button.disabled = true;
      button.setAttribute("aria-busy", "true");
    });
  });
});
