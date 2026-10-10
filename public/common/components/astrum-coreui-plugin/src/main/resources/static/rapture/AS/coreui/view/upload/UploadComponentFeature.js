
/*global Ext, AS*/

/**
 * @since 3.7
 */
Ext.define('AS.coreui.view.upload.UploadComponentFeature', {
  extend: 'AS.view.drilldown.Drilldown',
  alias: 'widget.as-coreui-uploadcomponentfeature',

  iconName: 'upload-component-default',

  masters: [
    {xtype: 'as-coreui-upload-repository-list'},
    {xtype: 'as-coreui-upload-component'}
  ]
});
