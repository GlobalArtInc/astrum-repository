
/*global Ext, NX*/

/**
 * Copy window controller
 *
 * @since 3.0
 */
Ext.define('NX.controller.Copy', {
  extend: 'NX.app.Controller',

  views: [
    'CopyWindow'
  ],

  refs: [
    {
      ref: 'copyModal',
      selector: 'nx-copywindow'
    }
  ],

  /**
   * @override
   */
  init: function () {
    var me = this;

    me.listen({
      component: {
        'nx-copywindow button[action=close]': {
          click: me.copyToClipboard
        }
      }
    });

    // Raw HTML button (not an Ext component). Must use a DOM-level delegate, not the component: listener domain.
    Ext.getBody().on('click', me.onCopyWidgetClick, me, { delegate: 'button.nx-copy-widget' });
  },

  copyToClipboard: function() {
    this.getCopyModal().close();
  },

  onCopyWidgetClick: function(e) {
    var button = e.getTarget('button.nx-copy-widget');
    if (!button) {
      return;
    }
    var copyText = button.getAttribute('data-copy-text'),
        repoFormat = button.getAttribute('data-repo-format');
    Ext.widget('nx-copywindow', {
      copyText: copyText,
      repoFormat: repoFormat
    });
  }
});
