export const getApiUrl = (endpoint: string): string => {
  const asterApiHost = String(window.appConfig.get('ASTER_API_HOST') || '');
  const cleanEndpoint = endpoint.startsWith('/') ? endpoint : `/${endpoint}`;
  return `${asterApiHost}${cleanEndpoint}`;
};
