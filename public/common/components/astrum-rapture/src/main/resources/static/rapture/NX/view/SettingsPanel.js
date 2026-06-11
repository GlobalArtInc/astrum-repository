
/*global Ext*/

/**
 * Abstract settings panel.
 *
 * @since 3.0
 */
Ext.define('NX.view.SettingsPanel', {
  extend: 'Ext.panel.Panel',
  alias: 'widget.nx-settingsPanel',
  autoScroll: true,

  cls: 'nx-hr',

  layout: {
    type: 'vbox',
    align: 'stretch'
  },

  // TODO maxWidth: 1024,

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = {
      xtype: 'panel',
      ui: 'nx-inset',

      items: me.settingsForm || []
    };

    me.callParent();
  },

  /**
   * @override
   * @param form The form to add to this settings panel
   */
  addSettingsForm: function(form) {
    if (this.isDestroyed || this.destroying || !this.items) {
      return;
    }
    var panel = this.down('panel');
    if (panel && !panel.isDestroyed && !panel.destroying) {
      panel.add(form);
    }
  },

  /**
   * Remove all settings forms from this settings panel.
   *
   * @override
   */
  removeAllSettingsForms: function() {
    if (this.isDestroyed || this.destroying || !this.items) {
      return;
    }
    var panel = this.down('panel');
    if (panel && !panel.isDestroyed && !panel.destroying) {
      panel.removeAll();
    }
  },

  /**
   * Loads an {@link Ext.data.Model} into this form
   * (internally just calls {@link NX.view.SettingsForm#loadRecord}).
   *
   * @public
   * @param model The model to load
   */
  loadRecord: function(model) {
    if (this.isDestroyed || this.destroying || !this.items) {
      return;
    }
    var settingsForm = this.down('nx-settingsform');
    if (settingsForm && !settingsForm.isDestroyed && !settingsForm.destroying) {
      settingsForm.loadRecord(model);
    }
  }

});
