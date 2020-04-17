<?php

declare(strict_types = 1);

use Behat\Mink\Exception\UnsupportedDriverActionException;
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
        throw new \Exception(
            sprintf(
                "The text '%s' was not found in any heading on the page %s",
                $heading,
                $this->getSession()->getCurrentUrl()
            )
        );
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
                    throw new \Exception(
                        sprintf(
                            "The text '%s' was found in a heading on the page %s",
                            $heading,
                            $this->getSession()->getCurrentUrl()
                        )
                    );
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

    /**
     * @Then I should see the link :link
     */
    public function assertLinkVisible($link)
    {
        $element = $this->getSession()->getPage();
        $result = $element->findLink($link);

        try {
            if ($result && !$result->isVisible()) {
                throw new \Exception(
                    sprintf("No link to '%s' on the page %s", $link, $this->getSession()->getCurrentUrl())
                );
            }
        } catch (UnsupportedDriverActionException $e) {
            // We catch the UnsupportedDriverActionException exception in case
            // this step is not being performed by a driver that supports javascript.
            // All other exceptions are valid.
        }

        if (empty($result)) {
            throw new \Exception(
                sprintf("No link to '%s' on the page %s", $link, $this->getSession()->getCurrentUrl())
            );
        }
    }

    /**
     * Links are not loaded on the page.
     *
     * @Then I should not see the link :link
     */
    public function assertNotLinkVisible($link)
    {
        $element = $this->getSession()->getPage();
        $result = $element->findLink($link);

        try {
            if ($result && $result->isVisible()) {
                throw new \Exception(
                    sprintf(
                        "The link '%s' was present on the page %s and was not supposed to be",
                        $link,
                        $this->getSession()->getCurrentUrl()
                    )
                );
            }
        } catch (UnsupportedDriverActionException $e) {
            // We catch the UnsupportedDriverActionException exception in case
            // this step is not being performed by a driver that supports javascript.
            // All other exceptions are valid.
        }

        if ($result) {
            throw new \Exception(
                sprintf(
                    "The link '%s' was present on the page %s and was not supposed to be",
                    $link,
                    $this->getSession()->getCurrentUrl()
                )
            );
        }
    }
}
