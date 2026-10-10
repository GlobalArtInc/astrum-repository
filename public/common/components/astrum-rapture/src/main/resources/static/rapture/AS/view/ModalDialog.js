
/*global Ext*/

/**
 * Custom modal dialog window.
 *
 * @since 3.0
 */
Ext.define('AS.view.ModalDialog', {
  extend: 'Ext.window.Window',
  alias: 'widget.as-modal-dialog',

  layout: 'fit',
  autoShow: true,
  modal: true,
  constrain: true,
  closable: true,
  resizable: false,

  // Standard modal widths
  statics: {
    SMALL_MODAL: 320,
    MEDIUM_MODAL: 480,
    LARGE_MODAL: 700
  },
  onHide: function() {
    var me = this, el;

    me.callParent(arguments);

    //we have to do a blur here, so that focus won't be forced back on the button that caused this window to open
    el = Ext.dom.Element.getActiveElement(true);
    if (el) {
      el.blur();
    }
  }
});
