
/*global Ext*/

/**
 * Unsaved changes window.
 *
 * @since 3.0
 */
Ext.define('NX.view.UnsavedChanges', {
  extend: 'NX.view.ModalDialog',
  requires: [
    'NX.I18n'
  ],
  alias: 'widget.nx-unsaved-changes',

  /**
   * Panel with content to be saved.
   *
   * @public
   */
  content: null,

  /**
   * Function to call if content is to be discarded.
   *
   * @public
   */
  callback: Ext.emptyFn,

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.title = NX.I18n.get('UnsavedChanges_Title');

    me.setWidth(NX.view.ModalDialog.SMALL_MODAL);

    Ext.apply(me, {
      items: {
        xtype: 'panel',
        ui: 'nx-inset',
        html: NX.I18n.get('UnsavedChanges_Help_HTML'),
        buttonAlign: 'left',
        buttons: [
          {
            text: NX.I18n.get('UnsavedChanges_Discard_Button'),
            ui: 'nx-primary',
            itemId: 'nx-discard',
            handler: function () {
              // Discard changes and load new content
              if (me.content) {
                me.content.resetUnsavedChangesFlag(true);
              }
              me.callback();
              me.close();
            }
          },
          { text: NX.I18n.get('UnsavedChanges_Back_Button'), handler: me.close, scope: me }
        ]
      }
    });

    me.on({
      resize: function() {
        me.down('#nx-discard').focus();
      },
      single: true
    });

    me.callParent();
  }

});
