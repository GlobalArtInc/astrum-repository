
/*global Ext, AS*/

/**
 * API feature flag store.
 *
 * @since 3.6
 */
Ext.define('AS.coreui.store.Api', {
    extend: 'Ext.data.Store',

    proxy: {
        type: 'direct',
        api: {
            read: 'AS.direct.coreui_Api.read'
        },

        reader: {
            type: 'json',
            rootProperty: 'data',
            successProperty: 'success'
        }
    }
});
