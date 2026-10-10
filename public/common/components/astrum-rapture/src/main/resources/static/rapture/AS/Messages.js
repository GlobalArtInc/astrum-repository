
/*global Ext, AS*/

/**
 * Helpers to interact with Message controller.
 *
 * @since 3.0
 */
Ext.define('AS.Messages', {
  singleton: true,

  requires: [
    'AS.State'
  ],

  info: function(message) {
    this.toast(message, 'info', 'fa-info');
  },

  success: function(message) {
    this.toast(message, 'success', 'fa-check-circle');
  },

  warning: function(message) {
    this.toast(message, 'warning', 'fa-exclamation-circle');
  },

  error: function(message) {
    this.toast(message, 'error', 'fa-exclamation-triangle');
  },

  /** @private */
  toast: function(message, type, iconCls) {
    // When the React shell is active, delegate to the React toast bridge so the
    // message is visible. The React shell suppresses .x-toast elements via CSS.
    if (typeof window !== 'undefined' && window.__astrumToast && typeof window.__astrumToast[type] === 'function') {
      window.__astrumToast[type](message);
      return;
    }
    const isFirst = !document.querySelector('.x-toast');
    const extraStyle = isFirst ? 'position: absolute; transform: translate(-24px, 70px);' : 'transform: translateY(70px)';
    setTimeout(function delay() {
      Ext.toast({
        baseCls: type,
        html:
            '<div role="presentation" class="icon x-fa ' + iconCls + '"></div>' +
            '<div class="text">' + Ext.htmlEncode(message) + '</div>' +
            '<div class="dismiss"><a aria-label="Dismiss" href="javascript:;" onclick="Ext.getCmp(this.closest(\'.x-toast\').id).close()"><i class="fa fa-times-circle"></i></a></div>',
        align: 'tr',
        style: extraStyle,
        stickOnClick: true,
        minWidth: 150,
        maxWidth: 400,
        autoCloseDelay: AS.State.getValue('messageDuration', 5000),
        slideInDuration: AS.State.getValue('animateDuration', 800),
        slideBackDuration: AS.State.getValue('animateDuration', 500),
        hideDuration: AS.State.getValue('animateDuration', 500),
        slideInAnimation: 'elasticIn',
        slideBackAnimation: 'elasticIn',
        ariaRole: 'alert'
      });
    }, 100);
  }
});
