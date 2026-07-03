

/*global Ext, NX*/

/*global NX*/

/**
 * 'panelMessage' factory.
 */
Ext.define('NX.coreui.view.formfield.factory.FormfieldPanelMessageFactory', {
  singleton: true,
  alias: ['nx.formfield.factory.panelMessage'],

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
          cls: 'nx-drilldown-' + panelType,
          iconCls: NX.Icons.cls('drilldown-' + panelType, 'x16'),
          ui: 'nx-drilldown-message',
          minHeight: 54,
          title: formField.helpText,
          listeners: {
            afterrender: function(panel) {
              var title = panel.el.down('.x-panel-header-title-nx-drilldown-message');
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
