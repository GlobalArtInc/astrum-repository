
/*global Ext, AS*/

/**
 * Assets grid.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.component.AssetList', {
  extend: 'AS.view.drilldown.Master',
  alias: 'widget.as-coreui-component-asset-list',
  requires: [
    'AS.I18n'
  ],
  mixins: {
    componentUtils: 'AS.coreui.mixin.ComponentUtils'
  },
  /**
   * Currently shown component model.
   */
  componentModel: undefined,

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    Ext.apply(me, {
      store: 'ComponentAsset',

      cls: 'as-hr',

      // Marker for source of targets to be shown in container assets
      assetContainerSource: true,

      // Prevent the store from automatically loading
      loadStore: Ext.emptyFn,

      allowDeselect: true,

      viewConfig: {
        emptyText: 'No assets found',
        deferEmptyText: false
      },

      columns: [
        {
          xtype: 'as-iconcolumn',
          dataIndex: 'name',
          width: 36,
          iconVariant: 'x16',
          iconName: function(value) {
            var icon = me.mixins.componentUtils.getIconForAssetName(value);
            if (icon) {
              var iconName = icon.get('name');
              if (iconName) {
                return iconName;
              }
            }
            return 'default';
          }
        },
        {
          text: AS.I18n.get('SearchResultAssetList_Name_Header'),
          dataIndex: 'name',
          flex: 2.5,
          renderer: Ext.htmlEncode
        }
      ]
    });

    me.callParent();
  },

  /**
   * @public
   *
   * Sets owning component.
   * @param {AS.coreui.model.Component} componentModel owning component
   */
  setComponentModel: function(componentModel) {
    var me = this;

    me.componentModel = componentModel;
    me.fireEvent('updated', me, me.componentModel);
  }

});
