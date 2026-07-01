
/*global Ext, NX*/

/**
 * Preview UI Settings controller. Registers the feature in the admin menu.
 */
Ext.define('NX.coreui.controller.PreviewUiSettings', {
  extend: 'NX.app.Controller',
  requires: [
    'NX.Permissions',
    'NX.I18n',
    'NX.State'
  ],

  /**
   * @override
   */
  init: function() {
    var me = this;

    me.getApplication().getFeaturesController().registerFeature({
      mode: 'admin',
      // Preserve the legacy bookmark token `admin/system/previewui` for existing routes and deep links.
      // TODO: Remove this ExtJS feature registration once the Classic UI is fully retired.
      path: '/System/Preview UI',
      text: NX.I18n.get('PreviewUiSettings_Text'),
      description: NX.I18n.get('PreviewUiSettings_Description'),
      view: {xtype: 'nx-coreui-react-main-container'},
      iconCls: 'x-fa fa-eye',
      visible: function() {
        return NX.State.getValue('astrum.previewui.enabled', false);
      },
      weight: 10
    });
  }
});

