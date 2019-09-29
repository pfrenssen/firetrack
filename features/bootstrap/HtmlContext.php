<?php

use Behat\MinkExtension\Context\RawMinkContext;

/**
 * Step definitions for interacting with HTML pages.
 */
class HtmlContext extends RawMinkContext
{

    /**
     * @Then I (should )see the heading :heading
     */
    public function assertHeading(string $heading): void
    {
        $element = $this->getSession()->getPage();
        foreach (['h1', 'h2', 'h3', 'h4', 'h5', 'h6'] as $tag) {
            $results = $element->findAll('css', $tag);
            foreach ($results as $result) {
                if ($result->getText() == $heading) {
                    return;
                }
            }
        }
        throw new \Exception(sprintf("The text '%s' was not found in any heading on the page %s",
            $heading, $this->getSession()->getCurrentUrl()));
    }

    /**
     * @Then I (should )not see the heading :heading
     */
    public function assertNoHeading(string $heading): void
    {
        $element = $this->getSession()->getPage();
        foreach (['h1', 'h2', 'h3', 'h4', 'h5', 'h6'] as $tag) {
            $results = $element->findAll('css', $tag);
            foreach ($results as $result) {
                if ($result->getText() == $heading) {
                    throw new \Exception(sprintf("The text '%s' was found in a heading on the page %s",
                        $heading, $this->getSession()->getCurrentUrl()));
                }
            }
        }
    }

    /**
     * Clicks the link with the given id|title|alt|text.
     *
     * @When I click :link
     */
    public function clickLink(string $link): void
    {
        $this->getSession()->getPage()->clickLink($link);
    }
}
