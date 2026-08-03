
/**
 * Copy text to clipboard with fallback support for browsers/contexts
 * where Clipboard API is not available.
 *
 * This function attempts to copy text using two methods:
 * 1. Modern Clipboard API (navigator.clipboard) - requires HTTPS or localhost
 * 2. execCommand fallback - works in insecure contexts (HTTP)
 *
 * @param {string} text - The text to copy to clipboard
 * @returns {Promise<boolean>} - Resolves to true on success, false on failure
 *
 * @remarks
 * The execCommand method is deprecated but remains the only option for HTTP contexts.
 * For best security and performance, deploy Astrum over HTTPS.
 */
export async function copyToClipboard(text) {
  // Validate input to prevent copying invalid values
  if (text == null || text === '') {
    console.warn('copyToClipboard called with empty/null text:', text);
    return false;
  }

  // Modern Clipboard API (requires HTTPS or localhost)
  if (navigator.clipboard && navigator.clipboard.writeText) {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch (err) {
      console.warn('Clipboard API failed, trying fallback:', err);
      // Fall through to fallback methods
    }
  }

  // Fallback: execCommand with textarea (works in HTTP contexts)
  const textarea = document.createElement('textarea');
  try {
    textarea.value = text;
    textarea.style.position = 'fixed';
    textarea.style.left = '-99999px';
    textarea.style.height = '1em';
    textarea.style.width = '1em';
    textarea.setAttribute('readonly', '');
    document.body.appendChild(textarea);
    textarea.select();

    const successful = document.execCommand('copy');
    return successful;
  } catch (err) {
    console.error('All clipboard methods failed:', err);
    return false;
  } finally {
    // Always clean up the textarea, even if execCommand throws
    document.body.removeChild(textarea);
  }
}

