
/*global Ext, AS*/

/**
 * Copy window controller
 *
 * @since 3.0
 */
Ext.define('AS.controller.Copy', {
  extend: 'AS.app.Controller',

  views: [
    'CopyWindow'
  ],

  refs: [
    {
      ref: 'copyModal',
      selector: 'as-copywindow'
    }
  ],

  /**
   * @override
   */
  init: function () {
    var me = this;

    me.listen({
      component: {
        'as-copywindow button[action=close]': {
          click: me.copyToClipboard
        }
      }
    });

    // Raw HTML button (not an Ext component). Must use a DOM-level delegate, not the component: listener domain.
    Ext.getBody().on('click', me.onCopyWidgetClick, me, { delegate: 'button.as-copy-widget' });
  },

  copyToClipboard: function() {
    this.getCopyModal().close();
  },

  onCopyWidgetClick: function(e) {
    var button = e.getTarget('button.as-copy-widget');
    if (!button) {
      return;
    }
    var copyText = button.getAttribute('data-copy-text'),
        repoFormat = button.getAttribute('data-repo-format');
    Ext.widget('as-copywindow', {
      copyText: copyText,
      repoFormat: repoFormat
    });
  }
});
