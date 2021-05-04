import React from 'react';
import axios from 'axios';
import { graphql } from 'gatsby';
import PropTypes from 'prop-types';

import { PureLayout as Layout } from '../components/Layout';
import { PureSEO as SEO } from '../components/SEO';

export default function Home({ data }) {
  const handleClick = async () => {
    try {
      const response = await axios({
        url: '.netlify/functions/rainbow?name=Nemo',
        method: 'POST',
        data: { name: 'nemo' },
      });
      console.log('Response: ', response);
    } catch (error) {
      if (error.response) {
        console.log('Server responded with non 2xx code: ', error.response.data);
      } else if (error.request) {
        console.log('No response received: ', error.request);
      } else {
        console.log('Error setting up response: ', error.message);
      }
    }
  };

  return (
    <>
      <SEO data={data} title="Home" metadescription="Climate - Gatsby v3 Starter for MDX Gatsby Blog" />
      <Layout data={data}>
        <>
          <header>
            <h1>Rainbow</h1>
          </header>
          <button type="submit" onClick={() => { handleClick(); }}>Invoke Serverless</button>
        </>
      </Layout>
    </>
  );
}

Home.propTypes = {
  data: PropTypes.shape({
    site: PropTypes.shape({
      buildTime: PropTypes.string,
    }),
  }).isRequired,
};

export const query = graphql`
  query Home {
    site {
      ...LayoutFragment
      ...SEOFragment
    }
  }
`;
