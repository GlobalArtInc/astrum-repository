
/*global Ext, NX*/

/**
 * @since 3.17
 */
Ext.define('NX.onboarding.controller.Onboarding', {
  extend: 'NX.wizard.Controller',
  requires: [
    'NX.Messages',
    'NX.I18n',
    'NX.onboarding.step.OnboardingStartStep',
    'NX.onboarding.step.CommunityEulaStep',
    'NX.onboarding.step.ChangeAdminPasswordStep',
    'NX.onboarding.step.ConfigureAnonymousAccessStep',
    'NX.onboarding.step.OnboardingCompleteStep',
    'NX.State'
  ],
  views: [
    'OnboardingWizard',
    'OnboardingStartScreen',
    'ChangeAdminPasswordScreen',
    'ConfigureAnonymousAccessScreen',
    'OnboardingCompleteScreen',
    'OnboardingModal',
    'CommunityEulaScreen'
  ],
  stores: [
    'Onboarding'
  ],

  /**
   * @override
   */
  init: function() {
    var me = this;

    me.callParent();

    me.listen({
      component: {
        'nx-onboarding-wizard': {
          closed: me.reset
        }
      },
      controller: {
        '#State': {
          changed: me.stateChanged,
          userAuthenticated: me.stateChanged
        }
      },
      store: {
        '#Onboarding': {
          load: me.itemsLoaded
        }
      }
    });
  },

  stateChanged: function() {
    // When the React onboarding wizard is enabled, defer to it and skip the ExtJS wizard entirely.
    if (NX.State.getValue('astrum.react.onboarding.enabled')) {
      return;
    }

    var isOnboardingRequired = NX.State.getValue('onboarding.required'),
        user = NX.State.getUser();
    if (isOnboardingRequired && user && user['administrator']) {
      this.loadItems();
    }
  },

  /**
   * @override
   */
  finish: function() {
    NX.State.setValue('onboarding.required', false);
    var results = Ext.ComponentQuery.query('nx-onboarding-modal');
    if (results && results.length) {
      results[0].close();
    }
  },

  loadItems: function() {
    var me = this,
    store = me.getStore('Onboarding');

    if (!store.isLoaded() && !store.isLoading()) {
      store.load();
    }
  },

  itemsLoaded: function (store, records, successful) {
    var me = this;
    me.registerStep('NX.onboarding.step.OnboardingStartStep');
    if (successful && Array.isArray(records)) {
      records.forEach(function(record) {
        me.registerStep('NX.onboarding.step.' + record.get('type') + 'Step');
      });
    }
    else {
      NX.Messages.error(NX.I18n.get('Onboarding_LoadStepsError'));
    }
    me.registerStep('NX.onboarding.step.OnboardingCompleteStep');

    Ext.widget('nx-onboarding-modal');

    me.load();
  }
});
