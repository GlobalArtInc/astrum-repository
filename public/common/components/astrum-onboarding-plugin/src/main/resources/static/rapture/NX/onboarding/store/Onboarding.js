
/*global Ext, NX*/

/**
 * @since 3.17
 */
Ext.define('NX.onboarding.store.Onboarding', {
  extend: 'Ext.data.Store',
  model: 'NX.onboarding.model.Onboarding',

  proxy: {
    type: 'rest',
    url: 'service/rest/internal/ui/onboarding'
  }
});
