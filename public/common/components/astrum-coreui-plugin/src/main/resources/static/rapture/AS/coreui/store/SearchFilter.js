
/*global Ext, AS*/

/**
 * Search Filter store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.SearchFilter', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.SearchFilter',

  autoLoad: true,

  proxy: {
    type: 'memory',
    reader: {
      type: 'json'
    }
  }

});
