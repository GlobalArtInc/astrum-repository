
/*global Ext, AS*/

/**
 * Component/Asset tree model.
 *
 * @since 3.6
 */
Ext.define('AS.coreui.model.ComponentAssetTree', {
  extend: 'Ext.data.Model',
  mixins: {
    componentUtils: 'AS.coreui.mixin.ComponentUtils'
  },
  fields: [
    {name: 'id', type: 'string', sortType: 'asUCText'},
    {name: 'type', type: 'string', sortType: 'asUCText'},
    {name: 'text', type: 'string', convert: Ext.util.Format.htmlEncode},
    {name: 'iconCls', type: 'string', convert: function(value, record){
        var icon = record.mixins.componentUtils.getIconForAsset(record);
        if (icon) {
          return icon.get('cls');
        }
    }},
    {name: 'leaf', type: 'boolean'},
    {name: 'componentId', type: 'string'},
    {name: 'assetId', type: 'string'},
    {name: 'vulnerable', type: 'boolean'},
    {name: 'packageUrl', type: 'string'}
  ]
});
