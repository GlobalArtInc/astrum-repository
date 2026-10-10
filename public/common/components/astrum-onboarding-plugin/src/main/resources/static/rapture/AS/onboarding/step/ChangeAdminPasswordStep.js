
/*global Ext, AS*/

/**
 * @since 3.17
 */
Ext.define('AS.onboarding.step.ChangeAdminPasswordStep', {
  extend: 'AS.wizard.Step',
  requires: [
    'AS.onboarding.view.ChangeAdminPasswordScreen',
    'AS.State'
  ],

  config: {
    screen: 'AS.onboarding.view.ChangeAdminPasswordScreen',
    enabled: true
  },

  /**
   * @override
   */
  init: function () {
    var me = this;

    me.control({
      'button[action=next]': {
        click: me.changePassword
      },
      'button[action=back]': {
        click: me.moveBack
      }
    });
  },

  changePassword: function(button) {
    var me = this,
        password = button.up('form').down('#password').getValue();

    Ext.Ajax.request({
      url: AS.util.Url.relativePath + '/service/rest/internal/ui/onboarding/change-admin-password',
      method: 'PUT',
      params: password,
      success: function(){
        me.moveNext();
      },
      failure: function (response) {
        var message;

        try {
          message = JSON.parse(response.responseText);

          if (Array.isArray(message)) {
            message = message.map(function(e) { return e.message; }).join('\\n');
          }
        }
        catch (e) {
          message = response.statusText;
        }

        AS.Messages.error(message);
      }
    });
  }
});
