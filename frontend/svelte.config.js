import adapter from '@sveltejs/adapter-static';

export default {
  kit: {
    adapter: adapter({ pages: 'build', assets: 'build', fallback: '200.html', precompress: false, strict: false }),
    prerender: { entries: ['*'], handleHttpError: 'warn' }
  }
};
