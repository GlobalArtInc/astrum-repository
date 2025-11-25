
package org.globalart.astrum.testdb.example.template;

import org.globalart.astrum.datastore.api.DataAccess;
import org.globalart.astrum.datastore.api.Expects;
import org.globalart.astrum.datastore.api.SchemaTemplate;

@Expects({SpindleDAO.class, WidgetDAO.class})
@SchemaTemplate("material")
public interface SprocketDAO
    extends DataAccess
{
  // no additional behaviour
}
