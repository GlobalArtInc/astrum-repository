
/*global Ext, NX*/

/**
 * REST API
 *
 * @since 3.6
 */
Ext.define('NX.coreui.view.system.Api', {
  extend: 'Ext.container.Container',
  alias: 'widget.nx-rest-api',
  requires: [
    'NX.Icons'
  ],

  cls: 'nx-iframe-full',
  width: '100%',
  layout: 'fit'
});
