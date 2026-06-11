
/*global Ext*/

/**
 * Abstract add window.
 *
 * @since 3.0
 */
Ext.define('NX.view.AddPanel', {
  extend: 'Ext.panel.Panel',
  alias: 'widget.nx-addpanel',
  requires: [
    'NX.I18n'
  ],

  cls: 'nx-hr',

  layout: {
    type: 'vbox',
    align: 'stretch'
  },

  autoScroll: true,

  /**
   * @override
   */
  initComponent: function () {
    var me = this,
        addButton;

    // Create default buttons if they do not exist
    if (Ext.isDefined(me.settingsForm) && !Ext.isArray(me.settingsForm)) {
      if (!me.settingsForm.buttons) {
        me.settingsForm.buttons = [
          { text: NX.I18n.get('Add_Submit_Button'), action: 'add', ui: 'nx-primary', bindToEnter:  me.items.settingsFormSubmitOnEnter },
          { text: NX.I18n.get('Add_Cancel_Button'), handler: function () {
            this.up('nx-drilldown').showChild(0, true);
          }}
        ];
      }
    }

    // Add settings form to the panel
    me.items = {
      xtype: 'panel',
      ui: 'nx-inset',

      items: me.settingsForm
    };

    me.callParent();

    addButton = me.down('button[action=add]');
    if (addButton) {
      NX.Conditions.formIs(me.down('form'), function(form) {
        return !form.isDisabled() && form.isValid();
      }).on({
        satisfied: function() {
          this.enable();
        },
        unsatisfied: function() {
          this.disable();
        },
        scope: addButton
      });
    }
  }

});
