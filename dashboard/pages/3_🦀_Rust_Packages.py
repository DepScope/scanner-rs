"""
Rust Packages (Crates) Analysis Page
"""
import streamlit as st
import pandas as pd
import plotly.express as px

st.set_page_config(
    page_title="Rust Packages",
    page_icon="🦀",
    layout="wide"
)

def load_data(csv_path: str) -> pd.DataFrame:
    """Load and prepare the CSV data"""
    try:
        df = pd.read_csv(csv_path)
        df.columns = df.columns.str.strip()

        # Normalize column names
        if 'package_name' in df.columns:
            df['package'] = df['package_name']
        if 'has_version' in df.columns:
            df['version'] = df['has_version']
        if 'should_path' in df.columns and 'location' not in df.columns:
            df['location'] = df['should_path']
        elif 'application_root' in df.columns and 'location' not in df.columns:
            df['location'] = df['application_root']

        if 'parent_package' in df.columns:
            df['match_package'] = df['parent_package'].fillna('none')
        elif 'match_package' not in df.columns:
            df['match_package'] = 'none'

        return df
    except FileNotFoundError:
        st.error(f"CSV file not found: {csv_path}")
        st.stop()
    except Exception as e:
        st.error(f"Error loading CSV: {e}")
        st.stop()

def main():
    st.title("🦀 Rust Packages (Crates) Analysis")

    # Load data from uploaded file or path
    if 'uploaded_df' in st.session_state and st.session_state.uploaded_df is not None:
        df = st.session_state.uploaded_df.copy()
    else:
        csv_path = st.session_state.get('csv_path', '../output.csv')
        df = load_data(csv_path)

    # Normalize columns if needed
    if 'package_name' in df.columns and 'package' not in df.columns:
        df['package'] = df['package_name']
    if 'has_version' in df.columns and 'version' not in df.columns:
        df['version'] = df['has_version']
    if 'should_path' in df.columns and 'location' not in df.columns:
        df['location'] = df['should_path']

    # Filter for Rust packages using ecosystem column if available
    if 'ecosystem' in df.columns:
        rust_df = df[df['ecosystem'].str.lower() == 'rust'].copy()
    else:
        rust_df = pd.DataFrame()

    # If filtering resulted in empty dataframe, show message
    if len(rust_df) == 0:
        st.info("No Rust packages found in the dataset.")
        rust_df = pd.DataFrame()

    # Overview metrics
    st.header("📊 Rust Overview")
    col1, col2, col3, col4 = st.columns(4)

    with col1:
        st.metric("Total Rust Crates", len(rust_df))

    with col2:
        unique_packages = rust_df['package'].nunique() if 'package' in rust_df.columns else 0
        st.metric("Unique Crates", unique_packages)

    with col3:
        unique_locations = rust_df['location'].nunique() if 'location' in rust_df.columns else 0
        st.metric("Locations", unique_locations)

    with col4:
        if 'match_package' in rust_df.columns:
            infected = rust_df[rust_df['match_package'].str.lower() != 'none'].shape[0]
        else:
            infected = 0
        st.metric("⚠️ Infected", infected, delta_color="inverse")

    # Top 20 most used Rust crates
    st.header("🔝 Top 20 Most Used Rust Crates")

    if 'package' in rust_df.columns and len(rust_df) > 0:
        package_counts = rust_df['package'].value_counts().head(20)

        fig = px.bar(
            x=package_counts.values,
            y=package_counts.index,
            orientation='h',
            labels={'x': 'Occurrences', 'y': 'Crate Name'},
            title="Top 20 Rust Crates by Frequency",
            color=package_counts.values,
            color_continuous_scale='Oranges'
        )
        fig.update_layout(height=700, showlegend=False)
        st.plotly_chart(fig, width='stretch')

        # Show table with details
        st.subheader("Crate Details")
        top_packages = package_counts.head(20).index.tolist()

        # Group by package and show versions
        for package in top_packages[:5]:  # Show details for top 5
            with st.expander(f"🦀 {package} ({package_counts[package]} occurrences)"):
                pkg_df = rust_df[rust_df['package'] == package]
                if 'version' in pkg_df.columns:
                    version_counts = pkg_df['version'].value_counts()
                    col1, col2 = st.columns(2)

                    with col1:
                        st.write("**Versions:**")
                        for version, count in version_counts.items():
                            st.write(f"- `{version}`: {count} locations")

                    with col2:
                        st.write("**Sample Locations:**")
                        for loc in pkg_df['location'].head(3):
                            st.write(f"- {loc}")
    else:
        st.info("No Rust crates found in the dataset")

    # Version consistency analysis
    st.header("📊 Version Consistency")

    if 'package' in rust_df.columns and 'version' in rust_df.columns and len(rust_df) > 0:
        # Find packages with multiple versions
        version_diversity = rust_df.groupby('package')['version'].nunique().sort_values(ascending=False)
        inconsistent = version_diversity[version_diversity > 1].head(10)

        if len(inconsistent) > 0:
            st.warning(f"Found {len(version_diversity[version_diversity > 1])} crates with multiple versions")

            fig = px.bar(
                x=inconsistent.values,
                y=inconsistent.index,
                orientation='h',
                labels={'x': 'Number of Different Versions', 'y': 'Crate'},
                title="Top 10 Crates with Most Version Variations",
                color=inconsistent.values,
                color_continuous_scale='Reds'
            )
            fig.update_layout(height=400)
            st.plotly_chart(fig, width='stretch')
        else:
            st.success("✅ All crates have consistent versions!")

    # Common Rust crates detection
    st.header("🔧 Popular Crate Detection")

    if 'package' in rust_df.columns and len(rust_df) > 0:
        popular_crates = {
            'Serde': ['serde', 'serde_json'],
            'Tokio': ['tokio'],
            'Async Runtime': ['async-std', 'tokio', 'smol'],
            'CLI': ['clap', 'structopt'],
            'HTTP': ['reqwest', 'hyper', 'actix-web', 'axum'],
            'Regex': ['regex'],
            'Logging': ['log', 'env_logger', 'tracing'],
            'Error Handling': ['anyhow', 'thiserror']
        }

        detected = {}
        for category, crates in popular_crates.items():
            count = 0
            found_crates = []
            for crate in crates:
                matches = rust_df[rust_df['package'].str.lower() == crate]
                if len(matches) > 0:
                    count += len(matches)
                    found_crates.append(f"{crate} ({len(matches)})")
            if count > 0:
                detected[category] = (count, found_crates)

        if detected:
            col1, col2 = st.columns(2)
            with col1:
                st.write("**Detected Categories:**")
                for category, (count, crates) in detected.items():
                    st.write(f"**{category}**: {count} total")
                    for crate in crates:
                        st.write(f"  - {crate}")

            with col2:
                # Pie chart of categories
                categories = list(detected.keys())
                counts = [detected[cat][0] for cat in categories]
                fig = px.pie(
                    values=counts,
                    names=categories,
                    title="Distribution by Category"
                )
                st.plotly_chart(fig, width='stretch')

    # Raw data
    with st.expander("🔍 View Rust Crate Data"):
        st.dataframe(rust_df, width='stretch', height=400)

if __name__ == "__main__":
    main()
