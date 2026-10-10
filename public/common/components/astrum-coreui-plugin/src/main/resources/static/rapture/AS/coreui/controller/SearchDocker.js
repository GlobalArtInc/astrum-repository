
/*global Ext, AS*/

/**
 * Docker repository search contribution.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.controller.SearchDocker', {
  extend: 'AS.app.Controller',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  init: function () {
    var me = this,
        search = me.getController('AS.coreui.controller.Search');

    search.registerCriteria([
      {
        id: 'attributes.docker.imageName',
        group: AS.I18n.get('SearchDocker_Group'),
        config: {
          format: 'docker',
          fieldLabel: AS.I18n.get('SearchDocker_Image_Name_FieldLabel'),
          width: 300
        }
      },
      {
        id: 'attributes.docker.imageTag',
        group: AS.I18n.get('SearchDocker_Group'),
        config: {
          format: 'docker',
          fieldLabel: AS.I18n.get('SearchDocker_Image_Tag_FieldLabel')
        }
      },
      {
        id: 'attributes.docker.layerAncestry',
        group: AS.I18n.get('SearchDocker_Group'),
        config: {
          format: 'docker',
          fieldLabel: AS.I18n.get('SearchDocker_LayerId_FieldLabel'),
          width: 500
        }
      },
      {
        id: 'assets.attributes.docker.content_digest',
        group: AS.I18n.get('SearchDocker_Group'),
        config: {
          format: 'docker',
          fieldLabel: AS.I18n.get('SearchDocker_ContentDigest_FieldLabel'),
          width: 500
        }
      },
      {
        id: 'attributes.docker.os',
        group: AS.I18n.get('SearchDocker_Group'),
        config: {
          format: 'docker',
          fieldLabel: AS.I18n.get('SearchDocker_OS_FieldLabel')
        }
      },
      {
        id: 'attributes.docker.architecture',
        group: AS.I18n.get('SearchDocker_Group'),
        config: {
          format: 'docker',
          fieldLabel: AS.I18n.get('SearchDocker_Architecture_FieldLabel')
        }
      },
      {
        id: 'attributes.docker.labels',
        group: AS.I18n.get('SearchDocker_Group'),
        config: {
          format: 'docker',
          fieldLabel: AS.I18n.get('SearchDocker_Labels_FieldLabel')
        }
      },
      {
        id: 'attributes.docker.author',
        group: AS.I18n.get('SearchDocker_Group'),
        config: {
          format: 'docker',
          fieldLabel: AS.I18n.get('SearchDocker_Author_FieldLabel')
        }
      }
    ], me);

    search.registerFilter({
      id: 'docker',
      name: 'Docker',
      text: AS.I18n.get('SearchDocker_Text'),
      description: AS.I18n.get('SearchDocker_Description'),
      readOnly: true,
      criterias: [
        {id: 'format', value: 'docker', hidden: true},
        {id: 'attributes.docker.imageName'},
        {id: 'attributes.docker.imageTag'},
        {id: 'attributes.docker.layerAncestry'},
        {id: 'assets.attributes.docker.content_digest'},
        {id: 'attributes.docker.os'},
        {id: 'attributes.docker.architecture'},
        {id: 'attributes.docker.labels'},
        {id: 'attributes.docker.author'}
      ]
    }, me);
  }

});
