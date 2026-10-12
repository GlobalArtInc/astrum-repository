import { Navigate, useLocation, useParams } from 'react-router';
import { editAddress } from '../api';

export default function ProfileRedirect() {
  const { name = '' } = useParams();
  const location = useLocation();
  return <Navigate to={`${editAddress(name)}${location.search}`} replace />;
}
