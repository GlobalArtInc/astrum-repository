
/*global Ext, NX*/

/**
 * @since 3.7
 */
Ext.define('NX.coreui.view.upload.UploadComponentFeature', {
  extend: 'NX.view.drilldown.Drilldown',
  alias: 'widget.nx-coreui-uploadcomponentfeature',

  iconName: 'upload-component-default',

  masters: [
    {xtype: 'nx-coreui-upload-repository-list'},
    {xtype: 'nx-coreui-upload-component'}
  ]
});
