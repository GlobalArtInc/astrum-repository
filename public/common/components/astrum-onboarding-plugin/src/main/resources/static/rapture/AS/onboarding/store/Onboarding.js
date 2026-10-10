
/*global Ext, AS*/

/**
 * @since 3.17
 */
Ext.define('AS.onboarding.store.Onboarding', {
  extend: 'Ext.data.Store',
  model: 'AS.onboarding.model.Onboarding',

  proxy: {
    type: 'rest',
    url: 'service/rest/internal/ui/onboarding'
  }
});
