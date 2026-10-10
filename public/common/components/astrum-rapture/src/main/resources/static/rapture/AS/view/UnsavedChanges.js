
/*global Ext*/

/**
 * Unsaved changes window.
 *
 * @since 3.0
 */
Ext.define('AS.view.UnsavedChanges', {
  extend: 'AS.view.ModalDialog',
  requires: [
    'AS.I18n'
  ],
  alias: 'widget.as-unsaved-changes',

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

    me.title = AS.I18n.get('UnsavedChanges_Title');

    me.setWidth(AS.view.ModalDialog.SMALL_MODAL);

    Ext.apply(me, {
      items: {
        xtype: 'panel',
        ui: 'as-inset',
        html: AS.I18n.get('UnsavedChanges_Help_HTML'),
        buttonAlign: 'left',
        buttons: [
          {
            text: AS.I18n.get('UnsavedChanges_Discard_Button'),
            ui: 'as-primary',
            itemId: 'as-discard',
            handler: function () {
              // Discard changes and load new content
              if (me.content) {
                me.content.resetUnsavedChangesFlag(true);
              }
              me.callback();
              me.close();
            }
          },
          { text: AS.I18n.get('UnsavedChanges_Back_Button'), handler: me.close, scope: me }
        ]
      }
    });

    me.on({
      resize: function() {
        me.down('#as-discard').focus();
      },
      single: true
    });

    me.callParent();
  }

});
