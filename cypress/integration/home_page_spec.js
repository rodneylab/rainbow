describe('/ accessibility checks', () => {
  it('passes accessibility checks', () => {
    cy.visit('/');
    cy.injectAxe();
    cy.log('Page header is rendered');
    cy.findByRole('heading', { name: /Rainbow Contrast Checker/i }).should('be.visible');
    cy.checkAccessibility();
  });
});
