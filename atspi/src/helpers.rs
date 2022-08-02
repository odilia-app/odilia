/*
TODO: generic version of new function
pub async fn new<'a, T>(conn: Connection, dest: String, path: OwnedObjectPath) -> zbus::Result<T<'a>> 
  where T: dyn Pxy
{
  T::builder(conn)
    .destination(dest)?
    .path(path)?
    .build()
    .await
}
*/
