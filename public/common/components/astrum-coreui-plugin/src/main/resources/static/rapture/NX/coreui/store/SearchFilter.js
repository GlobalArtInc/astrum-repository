
/*global Ext, NX*/

/**
 * Search Filter store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.SearchFilter', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.SearchFilter',

  autoLoad: true,

  proxy: {
    type: 'memory',
    reader: {
      type: 'json'
    }
  }

});
