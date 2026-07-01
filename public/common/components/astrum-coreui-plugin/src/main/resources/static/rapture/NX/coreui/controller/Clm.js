
/*global Ext, NX*/

/**
 * CLM controller.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.controller.Clm', {
  extend: 'NX.app.Controller',
  requires: [
    'NX.State',
    'NX.Messages',
    'NX.Permissions',
    'NX.Windows',
    'NX.I18n'
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
      selector: 'nx-coreui-clm-settings'
    },
    {
      ref: 'openDashboardButton',
      selector: 'nx-coreui-clm-settings button[action=open]'
    },
    {
      ref: 'showLinkCheckbox',
      selector: 'nx-coreui-clm-settings checkbox[name=showLink]'
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
        'nx-coreui-clm-settings button[action=verify]': {
          click: me.verifyConnection
        },
        'nx-coreui-clm-settings nx-settingsform': {
          submitted: me.onSubmitted
        },
        'nx-coreui-clm-settings button[action=open]': {
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

    NX.direct.clm_CLM.verifyConnection(values, function (response) {
      form.getEl().unmask();
      if (Ext.isDefined(response) && response.success) {
        NX.Messages.success(NX.I18n.format('Clm_Connection_Success', values.url));
        Ext.widget('nx-coreui-clm-settings-testresults', {applications: response.data});
      }
    });
  },

  /**
   * @private
   */
  onSubmitted: function(form, action) {
    NX.State.setValue('clm', Ext.apply(Ext.clone(NX.State.getValue('clm', {})), {
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
    var clmState = NX.State.getValue('clm'),
        openDashboardButton = this.getOpenDashboardButton(),
        enableOpenDashboardButton = NX.Permissions.check('astrum:settings:read') && clmState && clmState.enabled && clmState.url;

    // when Capability is modified but page isn't loaded
    if (!openDashboardButton) {
      return;
    }

    if (enableOpenDashboardButton) {
      openDashboardButton.enable();
    } else {
      openDashboardButton.disableWithTooltip(NX.I18n.get('Clm_Dashboard_Disabled_Tooltip'));
    }
  },

  /**
   * @private
   */
  openDashboardWindow: function() {
    var state = NX.State.getValue('clm');

    NX.Windows.open(state.url);
  }

});
