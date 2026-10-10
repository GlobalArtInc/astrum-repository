
/*global Ext, AS*/

/**
 * Preview UI Settings controller. Registers the feature in the admin menu.
 */
Ext.define('AS.coreui.controller.PreviewUiSettings', {
  extend: 'AS.app.Controller',
  requires: [
    'AS.Permissions',
    'AS.I18n',
    'AS.State'
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
      text: AS.I18n.get('PreviewUiSettings_Text'),
      description: AS.I18n.get('PreviewUiSettings_Description'),
      view: {xtype: 'as-coreui-react-main-container'},
      iconCls: 'x-fa fa-eye',
      visible: function() {
        return AS.State.getValue('astrum.previewui.enabled', false);
      },
      weight: 10
    });
  }
});

