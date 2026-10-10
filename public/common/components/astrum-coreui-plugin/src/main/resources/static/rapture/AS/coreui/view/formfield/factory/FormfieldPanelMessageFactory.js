

/*global Ext, AS*/

/*global AS*/

/**
 * 'panelMessage' factory.
 */
Ext.define('AS.coreui.view.formfield.factory.FormfieldPanelMessageFactory', {
  singleton: true,
  alias: ['as.formfield.factory.panelMessage'],

  create: function(formField) {
    var panelType = formField.attributes['panelType'];
    if (!panelType) {
      panelType = 'info';
    }

    const item = {
      xtype: 'container',
      layout: {
        type: 'vbox',
        align: 'stretch'
      },
      items: [
        {
          // Label
          xtype: 'component',
          html: formField.label,
          htmlDecode: true,
          style: {
            fontWeight: 'bold',
          },
          listeners: {
            afterrender: function(component) {
              component.addCls('x-unselectable');
            }
          }
        },
        {
          // Panel
          xtype: 'panel',
          itemId: panelType,
          cls: 'as-drilldown-' + panelType,
          iconCls: AS.Icons.cls('drilldown-' + panelType, 'x16'),
          ui: 'as-drilldown-message',
          minHeight: 54,
          title: formField.helpText,
          listeners: {
            afterrender: function(panel) {
              var title = panel.el.down('.x-panel-header-title-as-drilldown-message');
              if (title) {
                title.el.setStyle('font-weight', 'normal');
              }
            }
          }
        }
      ],
      validate: function() {
      },
    };
    return item;
  }
});
