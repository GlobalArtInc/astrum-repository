
/**
 * @since 3.28
 */
Ext.define('NX.coreui.controller.SearchConan', {
  extend: 'NX.app.Controller',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  init: function() {
    var me = this,
        search = me.getController('NX.coreui.controller.Search');

    search.registerCriteria([
      {
        id: 'attributes.conan.baseVersion',
        group: NX.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: NX.I18n.get('SearchConan_BaseVersion_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'attributes.conan.channel',
        group: NX.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: NX.I18n.get('SearchConan_Channel_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'attributes.conan.revision',
        group: NX.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: NX.I18n.get('SearchConan_RecipeRevision_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'assets.attributes.conan.packageId',
        group: NX.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: NX.I18n.get('SearchConan_PackageId_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'assets.attributes.conan.packageRevision',
        group: NX.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: NX.I18n.get('SearchConan_PackageRevision_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'attributes.conan.baseVersion.strict',
        group: NX.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: NX.I18n.get('SearchConan_BaseVersionStrict_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'attributes.conan.revision.latest',
        group: NX.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: NX.I18n.get('SearchConan_RecipeRevisionLatest_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'attributes.conan.settings.arch',
        group: NX.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: NX.I18n.get('SearchConan_Arch_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'attributes.conan.settings.os',
        group: NX.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: NX.I18n.get('SearchConan_Os_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'attributes.conan.settings.compiler',
        group: NX.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: NX.I18n.get('SearchConan_Compiler_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'attributes.conan.settings.compiler.version',
        group: NX.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: NX.I18n.get('SearchConan_CompilerVersion_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'attributes.conan.settings.compiler.runtime',
        group: NX.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: NX.I18n.get('SearchConan_CompilerRuntime_FieldLabel'),
          width: 250
        }
      }
    ], me);

    search.registerFilter({
      id: 'conan',
      name: 'Conan',
      text: NX.I18n.get('SearchConan_Text'),
      description: NX.I18n.get('SearchConan_Description'),
      readOnly: true,
      criterias: [
        {id: 'format', value: 'conan', hidden: true},
        {id: 'name.raw'},
        {id: 'attributes.conan.baseVersion'},
        {id: 'attributes.conan.baseVersion.strict'},
        {id: 'attributes.conan.channel'},
        {id: 'attributes.conan.revision'},
        {id: 'assets.attributes.conan.packageId'},
        {id: 'assets.attributes.conan.packageRevision'}
      ]
    }, me);
  }
});
