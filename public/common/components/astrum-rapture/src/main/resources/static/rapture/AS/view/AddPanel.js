
/*global Ext*/

/**
 * Abstract add window.
 *
 * @since 3.0
 */
Ext.define('AS.view.AddPanel', {
  extend: 'Ext.panel.Panel',
  alias: 'widget.as-addpanel',
  requires: [
    'AS.I18n'
  ],

  cls: 'as-hr',

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
          { text: AS.I18n.get('Add_Submit_Button'), action: 'add', ui: 'as-primary', bindToEnter:  me.items.settingsFormSubmitOnEnter },
          { text: AS.I18n.get('Add_Cancel_Button'), handler: function () {
            this.up('as-drilldown').showChild(0, true);
          }}
        ];
      }
    }

    // Add settings form to the panel
    me.items = {
      xtype: 'panel',
      ui: 'as-inset',

      items: me.settingsForm
    };

    me.callParent();

    addButton = me.down('button[action=add]');
    if (addButton) {
      AS.Conditions.formIs(me.down('form'), function(form) {
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
