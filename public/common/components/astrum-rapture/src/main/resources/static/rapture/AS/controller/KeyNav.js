
/*global Ext*/

/**
 * KeyNav controller.
 *
 * @since 3.0
 */
Ext.define('AS.controller.KeyNav', {
  extend: 'AS.app.Controller',
  requires: [
    'Ext.util.KeyNav',
    'Ext.dom.Element'
  ],

  /**
   * @override
   */
  init: function () {
    var me = this;

    me.listen({
      component: {
        'form button[bindToEnter=true]': {
          afterrender: me.installEnterKey
        }
      }
    });
  },

  /**
   * Install a key nav that will trigger click on any form buttons marked with "bindToEnter: true",
   * (usually submit button) on ENTER.
   *
   * @private
   */
  installEnterKey: function (button) {
    var form = button.up('form');

    button.keyNav = Ext.create('Ext.util.KeyNav', {
      target: form.el,
      enter: function () {
        if (!button.isDisabled()) {
          button.fireEvent('click', button);
        }
      }
    });
  }
});
