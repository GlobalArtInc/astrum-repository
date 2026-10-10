
/*global Ext, AS*/

/**
 * Clipboard related utils.
 *
 * @since 3.15
 */
Ext.define('AS.util.Clipboard', {
  singleton: true,

  /**
   * Copy specified text to clipboard
   *
   * @public
   * @param {String} text
   */
  copyToClipboard: function(text) {
    var textarea;
    if (navigator.clipboard) {
      navigator.clipboard.writeText(text);
    } else if (window.clipboardData && window.clipboardData.setData) {
      window.clipboardData.setData("Text", text);
    } else {
      textarea = document.createElement("textarea");
      textarea.value = text;
      textarea.style.position = 'fixed';
      textarea.style.left = '-99999px';
      textarea.style.height = '1em';
      textarea.style.width = '1em';
      document.body.appendChild(textarea);
      textarea.select();

      try {
        document.execCommand('copy');
      } catch (err) {
        console.error('Unable to copy text to clipboard: ', err);
      }

      document.body.removeChild(textarea);
    }
  }
});
