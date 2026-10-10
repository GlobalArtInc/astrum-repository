
/*global Ext, AS*/

/**
 * CLM controller.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.controller.Clm', {
  extend: 'AS.app.Controller',
  requires: [
    'AS.State',
    'AS.Messages',
    'AS.Permissions',
    'AS.Windows',
    'AS.I18n'
  ],

  stores: [
    'ClmApplication'
  ],
  views: [
    'clm.ClmSettings',
    'clm.ClmSettingsTestResults'
  ],
  refs: [
    {
      ref: 'clmSettingsForm',
      selector: 'as-coreui-clm-settings'
    },
    {
      ref: 'openDashboardButton',
      selector: 'as-coreui-clm-settings button[action=open]'
    },
    {
      ref: 'showLinkCheckbox',
      selector: 'as-coreui-clm-settings checkbox[name=showLink]'
    }
  ],

  /**
   * @override
   */
  init: function () {
    var me = this;

    me.listen({
      controller: {
        '#State': {
          clmchanged: me.onClmStateChanged
        }
      },
      component: {
        'as-coreui-clm-settings button[action=verify]': {
          click: me.verifyConnection
        },
        'as-coreui-clm-settings as-settingsform': {
          submitted: me.onSubmitted
        },
        'as-coreui-clm-settings button[action=open]': {
          afterrender: me.onClmStateChanged,
          click: me.openDashboardWindow
        }
      }
    });
  },

  /**
   * Verify the connection to the IQ Server.
   * @private
   */
  verifyConnection: function (button) {
    var form = button.up('form'),
        values = form.getForm().getFieldValues();

    form.getEl().mask('Checking connection to ' + values.url);

    AS.direct.clm_CLM.verifyConnection(values, function (response) {
      form.getEl().unmask();
      if (Ext.isDefined(response) && response.success) {
        AS.Messages.success(AS.I18n.format('Clm_Connection_Success', values.url));
        Ext.widget('as-coreui-clm-settings-testresults', {applications: response.data});
      }
    });
  },

  /**
   * @private
   */
  onSubmitted: function(form, action) {
    AS.State.setValue('clm', Ext.apply(Ext.clone(AS.State.getValue('clm', {})), {
      enabled: action.result.data.enabled,
      url: action.result.data.url,
      showLink: action.result.data.showLink,
      hasFirewall: action.result.data.hasFirewall
    }));
  },

  /**
   * Enable/disable the openDashboardButton
   * @private
   */
  onClmStateChanged: function () {
    var clmState = AS.State.getValue('clm'),
        openDashboardButton = this.getOpenDashboardButton(),
        enableOpenDashboardButton = AS.Permissions.check('astrum:settings:read') && clmState && clmState.enabled && clmState.url;

    // when Capability is modified but page isn't loaded
    if (!openDashboardButton) {
      return;
    }

    if (enableOpenDashboardButton) {
      openDashboardButton.enable();
    } else {
      openDashboardButton.disableWithTooltip(AS.I18n.get('Clm_Dashboard_Disabled_Tooltip'));
    }
  },

  /**
   * @private
   */
  openDashboardWindow: function() {
    var state = AS.State.getValue('clm');

    AS.Windows.open(state.url);
  }

});
