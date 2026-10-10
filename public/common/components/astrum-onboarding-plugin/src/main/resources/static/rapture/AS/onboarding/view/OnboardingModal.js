
/*global Ext, AS*/

/**
 * @since 3.17
 */
Ext.define('AS.onboarding.view.OnboardingModal', {
  extend: 'AS.view.ModalDialog',
  alias: 'widget.as-onboarding-modal',
  initComponent: function() {
    var me = this;

    Ext.apply(me, {
      header: false,
      closable: false,
      width: AS.view.ModalDialog.LARGE_MODAL,
      items: [{
        xtype: 'as-onboarding-wizard'
      }]
    });

    me.callParent();
    me.center();
  }
});
