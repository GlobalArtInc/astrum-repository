
/*global Ext, NX*/

/**
 * @since 3.17
 */
Ext.define('NX.onboarding.view.OnboardingModal', {
  extend: 'NX.view.ModalDialog',
  alias: 'widget.nx-onboarding-modal',
  initComponent: function() {
    var me = this;

    Ext.apply(me, {
      header: false,
      closable: false,
      width: NX.view.ModalDialog.LARGE_MODAL,
      items: [{
        xtype: 'nx-onboarding-wizard'
      }]
    });

    me.callParent();
    me.center();
  }
});
