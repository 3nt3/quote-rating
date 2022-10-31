import React from "react";
import { NextFunctionComponent } from "next";
import styles from "../styles/Quote.module.scss";
import Image from "next/image";

type Props = {
  quote: Quote;
};

type Quote = {
  avatar?: any;
  avatar_url?: string;
  id: number;
  content: string;
  username: string;
  author_id: number;
  sent_at: number;
  score: number;
};

export default function Quote(props: Props): NextFunctionComponent {
  return (
    <div className={styles.quote}>
      <picture>
        <img
          className={styles.quoteImg}
          src={props.quote.avatar_url}
          alt="adsf"
        />
      </picture>
      <div className={styles.quoteText}>
        <p className={styles.quoteUser}>
          {props.quote.username}{" "}
          <span className={styles.quoteDate}>Yesterday at 10:38</span>
        </p>
        <div className={styles.quoteQuote}>
          <div className={styles.quoteBlockQuoteThingy}></div>{" "}
          <p className={styles.quoteQuoteText}>
            {props.quote.content.replace(">", "").split("\n")[0]}
          </p>
        </div>
        <p>
          {
            props.quote.content.replace(/[-–]/, "—").split("\n")[
              props.quote.content.split("\n").length - 1
            ]
          }
        </p>
      </div>
    </div>
  );
}
