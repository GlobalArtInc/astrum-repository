
/*global Ext, AS*/

/**
 * @since 3.77
 */
Ext.define('AS.onboarding.step.CommunityEulaStep', {
  extend: 'AS.wizard.Step',
  requires: [
    'AS.onboarding.view.CommunityEulaScreen'
  ],

  config: {
    screen: 'AS.onboarding.view.CommunityEulaScreen',
    enabled: true
  },

  /**
   * @override
   */
  init: function () {
    const me = this;

    me.control({
      'button[action=next]': {
        click: me.agree
      },
      'button[action=back]': {
        click: me.moveBack
      }
    });
  },

  agree: function() {
    const me = this;

    var response = Ext.Ajax.request({
      url: AS.util.Url.relativePath + '/service/rest/v1/system/eula',
      method: 'GET',
      headers: {'Content-Type': 'application/json'}
    });
    response.then(function success(response) {
      var getResponse = JSON.parse(response.responseText);
      getResponse.accepted = true;

      var postOptions = {
        url: AS.util.Url.relativePath + '/service/rest/v1/system/eula',
        method: 'POST',
        jsonData: getResponse,
        success: function() {
          me.moveNext();
        },
        failure: function(response) {
          var message;

          try {
            message = JSON.parse(response.responseText);

            if (Array.isArray(message)) {
              message = message.map(function(e) { return e.message; }).join('\n');
            }
          } catch (e) {
            message = response.statusText;
          }

          AS.Messages.error(message);
        }
      };

      Ext.Ajax.request(postOptions);
    }, function error(response) {
      var message;

      try {
        message = JSON.parse(response.responseText);

        if (Array.isArray(message)) {
          message = message.map(function(e) { return e.message; }).join('\n');
        }
      } catch (e) {
        message = response.statusText;
      }

      AS.Messages.error(message);
    });
  }
});
