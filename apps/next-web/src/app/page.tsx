import site from '../config/site.json' assert { type: 'json' };

export default function Page() {
  return <main><h1>{site.title}</h1><p>{site.subtitle}</p></main>;
}
