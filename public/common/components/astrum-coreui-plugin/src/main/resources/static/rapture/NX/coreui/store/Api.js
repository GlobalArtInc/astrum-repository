
/*global Ext, NX*/

/**
 * API feature flag store.
 *
 * @since 3.6
 */
Ext.define('NX.coreui.store.Api', {
    extend: 'Ext.data.Store',

    proxy: {
        type: 'direct',
        api: {
            read: 'NX.direct.coreui_Api.read'
        },

        reader: {
            type: 'json',
            rootProperty: 'data',
            successProperty: 'success'
        }
    }
});
